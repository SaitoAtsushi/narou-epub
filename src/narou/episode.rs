pub use super::error::{Error, Result};
use crate::epub::Escape;
use crate::epub::NameId;
use crate::narou::{StatusCheck, unescape};
use regex::Regex;
use std::fmt::Display;
use unescape::Unescape;

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
            ImageType::Jpg => write!(f, "image/jpeg"),
            ImageType::Png => write!(f, "image/png"),
            ImageType::Gif => write!(f, "iamge/gif"),
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
        let corrected = matcher.replace_all(s, |captures: &regex::Captures<'_>| {
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
        let mut out = String::new();
        let mut image_urls = Vec::new();
        let mut last = 0;
        let re = Regex::new("<img src=\"([^\"]+)\" />").unwrap();
        let extract_extension = Regex::new("\\.([^.]+)$").unwrap();
        for caps in re.captures_iter(html) {
            let m = caps.get(0).unwrap();
            out.push_str(&html[last..m.start()]);
            let image_url = caps.get(1).unwrap().as_str().to_string();
            let image_url = "https:".to_string() + &image_url;
            let rel_image_url = minreq::get(image_url.as_str())
                .with_header("User-Agent", super::AGENT_NAME)
                .with_follow_redirects(false)
                .send()?
                .headers
                .get("location")
                .ok_or(Error::InvalidData)?
                .clone();
            let image_type = extract_extension
                .captures(&rel_image_url)
                .ok_or(Error::InvalidImageType)?
                .get(1)
                .ok_or(Error::InvalidImageType)?
                .as_str()
                .to_string();
            let image_body = minreq::get(&rel_image_url)
                .with_header("User-Agent", super::AGENT_NAME)
                .send()?
                .error_for_status()?
                .as_bytes()
                .to_vec();
            let image_name = format!("{}.{}", self.id.next().unwrap(), image_type);
            let image_tag = format!("<img src=\"{}\" />", image_name);
            image_urls.push(ImageInfo {
                name: image_name,
                image_type: image_type.parse()?,
                body: image_body,
            });
            out.push_str(&image_tag);
            last = m.end();
        }
        out.push_str(&html[last..]);
        Ok((out, image_urls))
    }

    fn try_next(&mut self) -> Result<Episode> {
        let uri = if self.series {
            format!("https://ncode.syosetu.com/{}/{}", self.ncode, self.cur)
        } else {
            format!("https://ncode.syosetu.com/{}", self.ncode)
        };
        let text = minreq::get(uri)
            .with_header("User-Agent", super::AGENT_NAME)
            .send()?
            .error_for_status()?
            .as_str()?
            .to_string();
        let matcher = Regex::new(if self.series {
            include_str!("extract.txt")
        } else {
            include_str!("short_extract.txt")
        })
        .unwrap();
        let captured = matcher.captures(&text).ok_or(Error::InvalidData)?;
        Ok(if self.series {
            let body = Self::correct(captured.get(3).ok_or(Error::InvalidData)?.as_str());
            let (body, images) = self.image_url_replace(&body)?;
            let chapter = captured.get(1).map(|x| regex::Match::as_str(&x).unescape());
            let title = captured
                .get(2)
                .ok_or(Error::InvalidData)?
                .as_str()
                .unescape();
            Episode {
                number: self.cur,
                chapter,
                title,
                body,
                series: self.series,
                images,
            }
        } else {
            let body = Self::correct(captured.get(1).ok_or(Error::InvalidData)?.as_str());
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
