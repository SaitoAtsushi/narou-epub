mod command;
mod epub;
mod narou;
use epub::{Epub, Escape, MediaType};
use indicatif::{ProgressBar, ProgressStyle};
use narou::episode::ImageInfo;
use sanitize_filename::sanitize;
use std::fs::File;
use std::thread;
use std::time::Duration;

use crate::epub::ReferenceType;
use crate::narou::episode::ImageType;

fn make_title_page(novel: &narou::Novel) -> String {
    format!(
        include_str!("title_page.txt"),
        novel.title().escape(),
        novel.author_name().escape()
    )
}

fn make_chapter(title: &str) -> String {
    format!(include_str!("chapter.txt"), title)
}

fn ncode_validate_and_normalize(s: &str) -> Option<String> {
    let valid_pattern = regex::Regex::new("(?i)^n[0-9]{4}[[:alpha:]]{0,3}$").unwrap();
    valid_pattern.is_match(s).then_some(s.to_lowercase())
}

fn image_type_to_media_type(it: ImageType) -> MediaType {
    match it {
        ImageType::Gif => MediaType::Gif,
        ImageType::Jpg => MediaType::Jpg,
        ImageType::Png => MediaType::Png,
    }
}

fn make_epub(ncode: &str, horizontal: bool, wait: f64) -> std::result::Result<(), narou::Error> {
    let ncode = ncode_validate_and_normalize(ncode).ok_or(narou::Error::InvalidNcode)?;
    let novel = narou::Novel::new(&ncode)?;
    let pb = ProgressBar::new(novel.episode().into()).with_message(novel.title().to_string());
    pb.set_style(ProgressStyle::with_template(
        "{msg}\n{spinner:.green} [{wide_bar:.cyan/blue}] {pos}/{len}",
    )?);
    let mut file = File::create(format!(
        "[{}] {}.epub",
        sanitize(novel.author_name()),
        sanitize(novel.title())
    ))
    .or(Err(narou::Error::EpubBuildFailed))?;
    let mut epub = Epub::new(&mut file)?;
    epub.set_source(format!("https://ncode.syosetu.com/{}/", ncode));
    epub.set_author(
        novel.author_name().to_string(),
        novel.author_yomigana().to_string(),
    );
    // builder.set_uuid(uuid);
    epub.set_title(novel.title().to_string());
    epub.set_modified(novel.last_update());
    epub.set_description(novel.story().to_string());
    epub.add_resource(
        "style.css",
        MediaType::Css,
        ReferenceType::Style,
        if horizontal {
            include_bytes!("horizontal_style.css")
        } else {
            include_bytes!("style.css")
        },
    )?;

    epub.set_direction(if horizontal {
        epub::Direction::Ltr
    } else {
        epub::Direction::Rtl
    });

    epub.add_content(
        "title.xhtml",
        "表題",
        MediaType::Xhtml,
        1,
        ReferenceType::Title,
        make_title_page(&novel).as_bytes(),
    )?;
    let mut prev_chapter: Option<String> = None;
    let mut chapter_number = 1;
    for i in novel.episodes()? {
        pb.inc(1);
        let mut episode = i?;
        if prev_chapter != episode.chapter {
            let chapter_title = episode
                .chapter
                .as_deref()
                .ok_or(narou::Error::InvalidData)?;
            epub.add_content(
                format!("chapter_{:04}.xhtml", chapter_number).as_str(),
                chapter_title,
                MediaType::Xhtml,
                1,
                ReferenceType::Text,
                make_chapter(chapter_title).as_bytes(),
            )?;
            chapter_number += 1;
            prev_chapter = episode.chapter.clone();
        };
        for ImageInfo {
            name,
            image_type,
            body,
        } in std::mem::take(&mut episode.images)
        {
            epub.add_resource(
                name.as_str(),
                image_type_to_media_type(ImageType::from(image_type)),
                ReferenceType::Image,
                &body,
            )?;
        }
        epub.add_content(
            format!("{:05}.xhtml", episode.number).as_str(),
            &episode.title,
            MediaType::Xhtml,
            if episode.chapter.is_none() { 1 } else { 2 },
            ReferenceType::Text,
            episode.to_string().as_bytes(),
        )?;
        thread::sleep(Duration::from_millis((wait * 1000.0) as u64));
    }
    epub.finish()?;
    pb.finish();
    Ok(())
}

fn main() {
    let cmd = match command::Cmd::parse() {
        Err(e) => {
            println!("{}", e);
            std::process::exit(2);
        }
        Ok(s) => s,
    };

    for ncode in cmd.ncodes {
        if let Err(x) = make_epub(&ncode, cmd.horizontal, cmd.wait) {
            println!("{}", x);
            std::process::exit(2);
        }
    }
}
