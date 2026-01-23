use super::Internet;
pub use super::error::{Error, Result};
use super::internet::Query;
use super::unescape::Unescape;
use crate::epub::Escape;
use crate::epub::NameId;
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
            _ => Err(Error::UnknownImageType),
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

trait TextUtil {
    fn head_and_next(&self) -> Option<(char, &str)>;
    fn between_and_next(&self, before: &str, after: &str) -> Option<(&str, &str)>;
    fn find_between_and_next(&self, before: &str, after: &str) -> Option<(&str, &str, &str)>;
}

impl TextUtil for str {
    // 文字列の最初の文字と最初の文字を除いたスライスを返す
    fn head_and_next(&self) -> Option<(char, &str)> {
        if self.is_empty() {
            None
        } else {
            let mut iter = self.char_indices();
            let (_, ch) = iter.next().unwrap();
            if let Some((next, _)) = iter.next() {
                Some((ch, &self[next..]))
            } else {
                Some((ch, &self[self.len()..]))
            }
        }
    }

    fn between_and_next(&self, before: &str, after: &str) -> Option<(&str, &str)> {
        let rest = self.strip_prefix(before)?;
        let (matched, rest) = rest.split_once(after)?;
        Some((matched, rest))
    }

    fn find_between_and_next(&self, before: &str, after: &str) -> Option<(&str, &str, &str)> {
        let (processed, rest) = self.split_once(before)?;
        let (center, rest) = rest.split_once(after)?;
        Some((processed, center, rest))
    }
}

impl EpisodeIter {
    fn correct(s: &str) -> String {
        let mut corrected = String::new();
        let mut rest = s;
        while !rest.is_empty() {
            let (ch, next) = rest.head_and_next().unwrap();
            if ch == '\n' {
                rest = next;
            } else if ch == '<' {
                if let Some((_, r)) = rest.between_and_next(r#"<p id="L"#, r#"">"#) {
                    corrected.push_str("<p>");
                    rest = r;
                } else if let Some((_, r)) = rest.between_and_next(r#"<a "#, ">") {
                    rest = r;
                } else if let Some(r) = rest.strip_prefix("<br>") {
                    corrected.push_str("<br/>");
                    rest = r;
                } else if let Some(r) = rest.strip_prefix("</a>") {
                    rest = r;
                } else if let Some((src, r)) = rest.between_and_next(r#"<img src=""#, r#"" "#) {
                    if let Some((_, r)) = r.split_once("/>") {
                        corrected.push_str(r#"<img src=""#);
                        corrected.push_str(src);
                        corrected.push_str(r#""/>"#);
                        rest = r;
                    } else {
                        corrected.push('<');
                        rest = next;
                    }
                } else {
                    corrected.push('<');
                    rest = next;
                }
            } else {
                corrected.push(ch);
                rest = next;
            }
        }

        corrected
    }

    fn image_url_replace(&mut self, html: &str) -> Result<(String, Vec<ImageInfo>)> {
        let internet = Internet::new()?;
        let mut out = String::new();
        let mut image_urls = Vec::new();
        let mut rest = html;
        loop {
            if let Some((processed, image_url, r)) =
                rest.find_between_and_next("<img src=\"", "\"/>")
            {
                let image_url = format!("https:{}", image_url);
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
                out.push_str(processed);
                out.push_str(&image_tag);
                rest = r;
            } else {
                out.push_str(rest);
                break;
            }
        }
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
