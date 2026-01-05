pub use super::error::{Error, Result};
use epub_builder::EpubContent;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::LOCATION;
use std::fmt::Display;
use std::io::Cursor;

pub enum ImageType {
    JPG,
    PNG,
    GIF,
}
pub struct Episode {
    pub number: u32,
    pub chapter: Option<String>,
    pub title: String,
    pub body: String,
    pub series: bool,
    pub images: Vec<(String, ImageType, Vec<u8>)>,
}

impl Display for ImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &ImageType::JPG => write!(f, "image/jpeg"),
            &ImageType::PNG => write!(f, "image/png"),
            &ImageType::GIF => write!(f, "iamge/gif"),
        }
    }
}

impl std::str::FromStr for ImageType {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "jpg" => Ok(ImageType::JPG),
            "png" => Ok(ImageType::PNG),
            "gif" => Ok(ImageType::GIF),
            _ => Err(Error::InvalidImageType),
        }
    }
}

pub struct EpisodeIter {
    pub(super) client: Client,
    pub(super) cur: u32,
    pub(super) max: u32,
    pub(super) series: bool,
    pub(super) ncode: String,
}

impl EpisodeIter {
    fn correct(s: &str) -> String {
        let matcher = Regex::new(
            "(?:(\n)|(<p id=\"L\\d+\">)|(<br>)|(<a [^>]*?>)|(</a>)|<img src=\"([^\"]+)\" [^/]*?/>)",
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

    fn image_url_replace(&self, html: &str) -> Result<(String, Vec<(String, ImageType, Vec<u8>)>)> {
        let mut out = String::new();
        let mut image_urls = Vec::new();
        let mut last = 0;
        let re = Regex::new("<img src=\"([^\"]+)\" />").unwrap();
        let mut counter = 0;
        for caps in re.captures_iter(html) {
            let m = caps.get(0).unwrap();
            out.push_str(&html[last..m.start()]);
            let image_url = caps.get(1).unwrap().as_str().to_string();
            let image_url = "https://".to_string() + &image_url;
            let rel_image_url = self
                .client
                .get(image_url.as_str())
                .send()?
                .headers()
                .get(LOCATION)
                .ok_or(Error::InvalidData)?
                .to_str()
                .or(Err(Error::FetchFailed))?
                .to_string();
            let re = Regex::new("\\.([^.]+)$").unwrap();
            let image_type = re
                .captures(&rel_image_url)
                .ok_or(Error::InvalidImageType)?
                .get(1)
                .ok_or(Error::InvalidImageType)?
                .as_str()
                .to_string();
            let image_body = self
                .client
                .get(&rel_image_url)
                .send()?
                .error_for_status()?
                .bytes()?
                .to_vec();
            let image_name = format!("{:05}_{:03}.{}", self.cur, counter, image_type);
            counter += 1;
            let image_tag = format!("<img src=\"{}\" />", image_name);
            image_urls.push((image_name, image_type.parse()?, image_body));
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
        let text = self.client.get(uri).send()?.error_for_status()?.text()?;
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
            let chapter = captured
                .get(1)
                .map(|x| html_escape::decode_html_entities(regex::Match::as_str(&x)).to_string());
            let title = html_escape::decode_html_entities(
                captured.get(2).ok_or(Error::InvalidData)?.as_str(),
            )
            .to_string();
            Episode {
                number: self.cur,
                chapter,
                title,
                body,
                series: self.series,
                images: images,
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
                images: images,
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
                html_escape::encode_text(&self.title),
                &self.body
            )
        } else {
            write!(f, include_str!("short_episode.txt"), &self.body)
        }
    }
}

impl From<Episode> for EpubContent<std::io::Cursor<String>> {
    fn from(episode: Episode) -> Self {
        if episode.series {
            EpubContent::new(
                format!("{:05}.xhtml", episode.number),
                std::io::Cursor::<String>::new(episode.to_string()),
            )
            .title(episode.title.clone())
            .level(if episode.chapter.is_none() { 1 } else { 2 })
        } else {
            EpubContent::new("body.xhtml", Cursor::<String>::new(episode.to_string()))
                .title("本文")
                .level(1)
        }
    }
}
