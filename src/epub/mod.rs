use std::fs::File;
use zip_builder::{Level, Result, ZipArchive};
mod escape;
mod id;
pub mod time;
pub use escape::Escape;
use id::{IdIter, ItemId};
use time::Time;
use uuid::Uuid;

#[derive(PartialEq)]
pub enum ReferenceType {
    Title,
    Text,
    Navi,
    Image,
    Style,
}

#[derive(PartialEq)]
pub enum MediaType {
    Css,
    Xhtml,
    Jpg,
    Png,
    Gif,
}

impl From<&MediaType> for &str {
    fn from(value: &MediaType) -> Self {
        match value {
            MediaType::Css => "text/css",
            MediaType::Xhtml => "application/xhtml+xml",
            MediaType::Jpg => "image/jpeg",
            MediaType::Png => "image/png",
            MediaType::Gif => "iamge/gif",
        }
    }
}

impl MediaType {
    fn as_str(&self) -> &'static str {
        self.into()
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

struct ContentMetadata {
    name: String,
    title: String,
    media_type: MediaType,
    reftype: ReferenceType,
    level: u32,
    id: ItemId,
}

struct ResourceMetadata {
    name: String,
    media_type: MediaType,
    reftype: ReferenceType,
    id: ItemId,
}

pub enum Direction {
    Rtl,
    Ltr,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Ltr => write!(f, "ltr"),
            Direction::Rtl => write!(f, "rtl"),
        }
    }
}

pub struct Epub<'a> {
    zip: ZipArchive<'a, File>,
    title: String,
    author: Option<(String, String)>,
    modified: Option<Time>,
    description: Option<String>,
    source: Option<String>,
    contents: Vec<ContentMetadata>,
    resources: Vec<ResourceMetadata>,
    direction: Direction,
    id_iter: IdIter,
}

struct Manifest<'a, 'b> {
    epub: &'a Epub<'b>,
}

impl<'a, 'b> std::fmt::Display for Manifest<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<manifest>")?;
        for x in self.epub.contents.iter() {
            if x.reftype == ReferenceType::Navi {
                write!(
                    f,
                    r#"<item media-type="{}" id="{}" href="{}" properties="nav"/>"#,
                    x.media_type, x.id, x.name
                )?;
            } else {
                write!(
                    f,
                    r#"<item media-type="{}" id="{}" href="{}"/>"#,
                    x.media_type, x.id, x.name
                )?;
            }
        }
        for x in self.epub.resources.iter() {
            if x.reftype == ReferenceType::Navi {
                write!(
                    f,
                    r#"<item media-type="{}" id="{}" href="{}" properties="nav"/>"#,
                    x.media_type, x.id, x.name
                )?;
            } else {
                write!(
                    f,
                    r#"<item media-type="{}" id="{}" href="{}"/>"#,
                    x.media_type, x.id, x.name
                )?;
            }
        }
        write!(f, "</manifest>")?;
        Ok(())
    }
}

struct Spine<'a, 'b> {
    epub: &'a Epub<'b>,
}

impl<'a, 'b> std::fmt::Display for Spine<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"<spine page-progression-direction="{}">"#,
            self.epub.direction
        )?;
        for x in self.epub.contents.iter() {
            write!(f, r#"<itemref idref="{}"/>"#, x.id)?;
        }
        write!(f, "</spine>")?;
        Ok(())
    }
}

struct Topic<'a, 'b> {
    epub: &'a Epub<'b>,
}

impl<'a, 'b> std::fmt::Display for Topic<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>目次</title></head><body><nav epub:type = "toc"><h1>目次</h1>"#
        )?;

        let mut level: u32 = 0;
        for i in self.epub.contents.as_slice() {
            if i.level > level {
                for _ in 0..(i.level - level) {
                    write!(f, "<ol>")?;
                }
                write!(f, "<li>")?;
            } else if i.level == level {
                write!(f, "</li><li>")?;
            } else if i.level < level {
                write!(f, "</li>")?;
                for _ in 0..(level - i.level) {
                    write!(f, "</ol></li>")?;
                }
                write!(f, "<li>")?;
            }

            write!(f, r#"<a href="{}">{}</a>"#, i.name, i.title.escape())?;
            level = i.level;
        }
        for _ in 0..level {
            write!(f, "</li></ol>")?;
        }

        write!(f, r#"</nav><nav epub:type = "landmarks"><ol>"#)?;

        for i in self.epub.contents.as_slice() {
            if i.reftype == ReferenceType::Title {
                write!(
                    f,
                    r#"<li><a epub:type="titlepage" href="{}">{}</a></li>"#,
                    i.name,
                    i.title.escape()
                )?;
            }
        }
        write!(f, r#"</ol></nav></body></html>"#)?;
        Ok(())
    }
}

impl<'a> Epub<'a> {
    pub fn new(file: &'a mut File) -> Result<Self> {
        let mut zip = ZipArchive::new(file);
        zip.add_entry("mimetype", b"application/epub+zip", Level::Raw)?;
        zip.add_entry(
            "META-INF/container.xml",
            include_bytes!("container.xml"),
            Level::High,
        )?;
        Ok(Epub {
            zip,
            title: String::new(),
            author: None,
            modified: None,
            description: None,
            source: None,
            contents: vec![],
            resources: vec![],
            direction: Direction::Rtl,
            id_iter: IdIter::new(),
        })
    }

    pub fn set_title(&mut self, title: String) -> &mut Self {
        self.title = title;
        self
    }

    pub fn set_author(&mut self, author: String, yomigana: String) -> &mut Self {
        self.author = Some((author, yomigana));
        self
    }

    pub fn set_modified(&mut self, modified: Time) -> &mut Self {
        self.modified = Some(modified);
        self
    }

    pub fn set_description(&mut self, description: String) -> &mut Self {
        self.description = Some(description);
        self
    }

    pub fn set_source(&mut self, source: String) -> &mut Self {
        self.source = Some(source);
        self
    }

    pub fn set_direction(&mut self, dir: Direction) -> &mut Self {
        self.direction = dir;
        self
    }

    pub fn add_content(
        &mut self,
        name: &str,
        title: &str,
        media_type: MediaType,
        level: u32,
        reftype: ReferenceType,
        body: &[u8],
    ) -> Result<&mut Self> {
        self.zip.add_entry(name, body, Level::High)?;
        self.contents.push(ContentMetadata {
            name: name.into(),
            title: title.into(),
            media_type,
            reftype,
            level,
            id: self.id_iter.next().unwrap(),
        });
        Ok(self)
    }

    pub fn add_resource(
        &mut self,
        name: &str,
        media_type: MediaType,
        reftype: ReferenceType,
        body: &[u8],
    ) -> Result<&mut Self> {
        self.zip.add_entry(name, body, Level::High)?;
        self.resources.push(ResourceMetadata {
            name: name.into(),
            media_type,
            reftype,
            id: self.id_iter.next().unwrap(),
        });
        Ok(self)
    }

    fn make_manifest(&self) -> Manifest<'_, '_> {
        Manifest { epub: self }
    }

    fn make_spine(&self) -> Spine<'_, '_> {
        Spine { epub: self }
    }

    fn make_topic(&self) -> Topic<'_, '_> {
        Topic { epub: self }
    }

    fn make_content(&self) -> String {
        let author = if let Some((ref author, ref yomigana)) = self.author {
            format!(
                r##"<dc:creator id="creator">{}</dc:creator><meta refines="#creator" property="role" scheme="marc:relators">aut</meta><meta refines="#creator" property="file-as">{}</meta>"##,
                author.escape(),
                yomigana.escape()
            )
        } else {
            "".to_string()
        };

        let source = if let Some(ref source) = self.source {
            let uuid = Uuid::new_v5(&Uuid::NAMESPACE_URL, source.as_bytes());
            format!(
                r#"<dc:identifier id="epub-id">urn:uuid:{}</dc:identifier><meta property="dcterms:source">{}</meta>"#,
                uuid, source
            )
        } else {
            "".to_string()
        };

        let modified = if let Some(ref modified) = self.modified {
            format!(r#"<meta property="dcterms:modified">{}</meta>"#, modified)
        } else {
            "".to_string()
        };

        let description = if let Some(ref description) = self.description {
            format!(
                r#"<dc:description>{}</dc:description>"#,
                description.escape()
            )
        } else {
            "".to_string()
        };

        format!(
            include_str!("content.txt"),
            source,
            self.title.escape(),
            "ja",
            author,
            modified,
            description,
            self.make_manifest(),
            self.make_spine()
        )
    }

    pub fn finish(&mut self) -> Result<()> {
        self.add_resource(
            "nav.xhtml",
            MediaType::Xhtml,
            ReferenceType::Navi,
            self.make_topic().to_string().as_bytes(),
        )?;
        self.zip
            .add_entry("content.opf", self.make_content().as_bytes(), Level::High)?;
        self.zip.flush()?;
        Ok(())
    }
}
