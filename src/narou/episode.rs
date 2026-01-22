use super::Internet;
pub use super::error::{Error, Result};
use super::internet::Query;
use super::unescape::Unescape;
use crate::epub::Escape;
use crate::epub::NameId;
use regex_lite::{Captures, Regex};
use std::fmt::Display;
use std::io::Read;

pub enum ImageType {
    Jpg,
    Png,
    Gif,
}

pub struct ImageInfo {
    pub name: String,
    pub image_type: ImageType,
    pub body: Vec<u8>,
}

pub struct Episode {
    #[allow(dead_code)]
    pub number: u32,
    pub chapter: Option<String>,
    pub title: String,
    pub body: String,
    pub series: bool,
    pub images: Vec<ImageInfo>,
}

impl Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ImageType::Jpg => write!(f, "jpg"),
            ImageType::Png => write!(f, "png"),
            ImageType::Gif => write!(f, "gif"),
        }
    }
}

impl std::str::FromStr for ImageType {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "jpg" => Ok(ImageType::Jpg),
            "png" => Ok(ImageType::Png),
            "gif" => Ok(ImageType::Gif),
            _ => Err(Error::InvalidImageType),
        }
    }
}

impl ImageType {
    fn from_extension(s: &str) -> Result<Self> {
        if s.ends_with(".jpg") {
            Ok(ImageType::Jpg)
        } else if s.ends_with(".png") {
            Ok(ImageType::Png)
        } else if s.ends_with(".gif") {
            Ok(ImageType::Gif)
        } else {
            Err(Error::UnknownImageType)
        }
    }
}

pub struct EpisodeIter {
    pub(super) cur: u32,
    pub(super) max: u32,
    pub(super) series: bool,
    pub(super) ncode: String,
    pub(super) id: crate::epub::IdIter<NameId>,
}

impl EpisodeIter {
    fn correct(s: &str) -> String {
        let matcher = Regex::new(
            "(?:(\n)|(<p id=\"L[0-9]+\">)|(<br>)|(<a [^>]*>)|(</a>)|<img src=\"([^\"]+)\" [^/]*/>)",
        )
        .unwrap();
        let corrected = matcher.replace_all(s, |captures: &Captures<'_>| {
            if captures.get(2).is_some() {
                "<p>".to_string()
            } else if captures.get(3).is_some() {
                "<br />".to_string()
            } else if captures.get(4).is_some() || captures.get(5).is_some() {
                "".to_string()
            } else if let Some(uri) = captures.get(6) {
                format!("<img src=\"{}\" />", uri.as_str())
            } else {
                "".to_string()
            }
        });
        corrected.to_string()
    }

    fn image_url_replace(&mut self, html: &str) -> Result<(String, Vec<ImageInfo>)> {
        let internet = Internet::new()?;
        let mut out = String::new();
        let mut image_urls = Vec::new();
        let mut last = 0;
        let re = Regex::new("<img src=\"([^\"]+)\" />").unwrap();
        for caps in re.captures_iter(html) {
            let m = caps.get(0).unwrap();
            out.push_str(&html[last..m.start()]);
            let image_url = caps.get(1).unwrap().as_str().to_string();
            let image_url = "https:".to_string() + &image_url;
            let rel_image_url = internet.open(image_url.as_str())?.header(Query::Location)?;
            let image_type = ImageType::from_extension(&rel_image_url)?;
            let mut response = internet.open(&rel_image_url)?.error_for_status()?;
            let mut image_body = Vec::<u8>::new();
            response.read_to_end(&mut image_body)?;
            let image_name = format!("{}.{}", self.id.next().unwrap(), image_type);
            let image_tag = format!("<img src=\"{}\" />", image_name);
            image_urls.push(ImageInfo {
                name: image_name,
                image_type,
                body: image_body,
            });
            out.push_str(&image_tag);
            last = m.end();
        }
        out.push_str(&html[last..]);
        Ok((out, image_urls))
    }

    fn extract(raw_html: &str) -> Option<(Option<&str>, &str, &str)> {
        let (chapter_title, rest) = match raw_html.split_once("<br>\n<span>") {
            Some((_, rest)) => rest.split_once("</span>").map(|x| (Some(x.0), x.1))?,
            None => (None, raw_html),
        };
        let (_, rest) = rest.split_once(r#"<h1 class="p-novel__title p-novel__title--rensai">"#)?;
        let (episode_title, rest) = rest.split_once("</h1>")?;
        let (_, rest) = rest.split_once(r#"<div class="js-novel-text p-novel__text">"#)?;
        let (body, _) = rest.split_once("</div>")?;

        Some((chapter_title, episode_title, body))
    }

    fn extract_short(raw_html: &str) -> Option<&str> {
        let (_, rest) = raw_html.split_once(r#"<div class="js-novel-text p-novel__text">"#)?;
        let (body, _) = rest.split_once("</div>")?;

        Some(body)
    }

    fn try_next(&mut self) -> Result<Episode> {
        let uri = if self.series {
            format!("https://ncode.syosetu.com/{}/{}", self.ncode, self.cur)
        } else {
            format!("https://ncode.syosetu.com/{}", self.ncode)
        };
        let internet = Internet::new()?;
        let mut text = String::new();
        internet
            .open(&uri)?
            .error_for_status()?
            .read_to_string(&mut text)?;
        Ok(if self.series {
            let (chapter, title, body) = Self::extract(&text).ok_or(Error::InvalidData)?;
            let body = Self::correct(body);
            let (body, images) = self.image_url_replace(&body)?;
            Episode {
                number: self.cur,
                chapter: chapter.map(|x| x.unescape()),
                title: title.unescape(),
                body,
                series: self.series,
                images,
            }
        } else {
            let body = Self::extract_short(&text).ok_or(Error::InvalidData)?;
            let body = Self::correct(body);
            let (body, images) = self.image_url_replace(&body)?;
            Episode {
                number: self.cur,
                chapter: None,
                title: "本文".to_string(),
                body,
                series: self.series,
                images,
            }
        })
    }
}

impl Iterator for EpisodeIter {
    type Item = Result<Episode>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur <= self.max {
            let episode = self.try_next();
            self.cur += 1;
            Some(episode)
        } else {
            None
        }
    }
}

impl Display for Episode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.series {
            write!(
                f,
                include_str!("episode.txt"),
                self.title.escape(),
                &self.body
            )
        } else {
            write!(f, include_str!("short_episode.txt"), &self.body)
        }
    }
}
