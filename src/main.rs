mod narou;
use clap::{Arg, ArgAction, Command, value_parser};
use epub_builder::{
    EpubBuilder, EpubContent, EpubVersion, MetadataOpfV3, ReferenceType, ZipLibrary,
};
use indicatif::{ProgressBar, ProgressStyle};
use sanitize_filename::sanitize;
use std::fs::File;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

fn make_title_page(novel: &narou::Novel) -> String {
    format!(
        include_str!("title_page.txt"),
        html_escape::encode_text(novel.title()),
        html_escape::encode_text(novel.author_name()),
        html_escape::encode_text(novel.story()),
    )
}

fn make_chapter(title: &str) -> String {
    format!(include_str!("chapter.txt"), title)
}

fn ncode_validate_and_normalize(s: &str) -> Option<String> {
    let valid_pattern = regex::Regex::new("(?i)^n\\d{4}[[:alpha:]]{0,3}$").unwrap();
    valid_pattern.is_match(s).then_some(s.to_lowercase())
}

fn make_epub(ncode: &str, horizontal: bool, wait: f64) -> std::result::Result<(), narou::Error> {
    let ncode = ncode_validate_and_normalize(ncode).ok_or(narou::Error::InvalidNcode)?;
    let uuid = Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("https://ncode.syosetu.com/{ncode}/").as_bytes(),
    );
    let novel = narou::Novel::new(&ncode)?;
    let pb = ProgressBar::new(novel.episode().into()).with_message(novel.title().to_string());
    pb.set_style(ProgressStyle::with_template(
        "{msg}\n{spinner:.green} [{wide_bar:.cyan/blue}] {pos}/{len}",
    )?);
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
    let file_as_meta = MetadataOpfV3 {
        property: "file-as".to_string(),
        dir: None,
        id: None,
        refines: Some("#epub-creator-0".to_string()),
        scheme: None,
        xml_lang: None,
        content: novel.author_yomigana().to_string(),
    };
    let source_page_meta = MetadataOpfV3 {
        property: "dcterms:source".to_string(),
        dir: None,
        id: None,
        refines: None,
        scheme: None,
        xml_lang: None,
        content: format!("https://ncode.syosetu.com/{}/", ncode),
    };

    builder.add_metadata_opf(Box::new(file_as_meta));
    builder.add_metadata_opf(Box::new(source_page_meta));
    builder.set_uuid(uuid);
    builder.set_title(novel.title());
    builder.set_authors(vec![novel.author_name().to_string()]);
    builder.set_lang("ja");
    builder.set_toc_name("目次");
    builder.set_modified_date(novel.last_update());
    builder.set_description(vec![novel.story().to_string()]);
    if horizontal {
        builder.stylesheet(include_bytes!("horizontal_style.css").as_slice())?;
    } else {
        builder.stylesheet(include_bytes!("style.css").as_slice())?;
    }
    if !horizontal {
        builder.metadata("direction", "rtl")?;
    }
    builder.epub_version(EpubVersion::V30);
    builder.add_content(
        EpubContent::new("title.xhtml", make_title_page(&novel).as_bytes())
            .title("表題")
            .reftype(ReferenceType::TitlePage),
    )?;
    let mut prev_chapter: Option<String> = None;
    let mut chapter_number = 1;
    for i in novel.episodes()? {
        pb.inc(1);
        let mut episode = i?;
        if prev_chapter != episode.chapter {
            builder.add_content(
                EpubContent::new(
                    format!("chapter_{:04}.xhtml", chapter_number),
                    make_chapter(
                        episode
                            .chapter
                            .as_deref()
                            .ok_or(narou::Error::InvalidData)?,
                    )
                    .as_bytes(),
                )
                .title(
                    episode
                        .chapter
                        .as_deref()
                        .ok_or(narou::Error::InvalidData)?,
                )
                .level(1),
            )?;
            chapter_number += 1;
            prev_chapter = episode.chapter.clone();
        };
        for (name, image_type, image_body) in std::mem::take(&mut episode.images) {
            builder.add_resource(name, image_body.as_slice(), image_type.to_string())?;
        }
        let content: EpubContent<std::io::Cursor<String>> = episode.into();
        builder.add_content(content)?;
        thread::sleep(Duration::from_millis((wait * 1000.0) as u64));
    }
    let file = File::create(format!(
        "[{}] {}.epub",
        sanitize(novel.author_name()),
        sanitize(novel.title())
    ))
    .or(Err(narou::Error::EpubBuildFailed))?;
    builder.generate(file)?;
    pb.finish();
    Ok(())
}

fn main() {
    let command_parser = Command::new("narou-epub")
        .disable_help_flag(true)
        .disable_version_flag(true)
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("help")
                .long("help")
                .short('h')
                .help("このヘルプを表示して終了します")
                .action(ArgAction::Help)
                .global(true),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .short('V')
                .help("バージョンを表示します")
                .action(ArgAction::Version)
                .global(true),
        )
        .arg(
            Arg::new("horizontal")
                .help("生成される EPUB が横書きになります")
                .long("horizontal")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("wait")
                .long("wait")
                .short('w')
                .value_name("秒")
                .value_parser(value_parser!(f64))
                .default_value("1.0"),
        )
        .arg(
            Arg::new("ncodes")
                .help("対象作品のNCODE")
                .action(ArgAction::Append)
                .value_name("NCODE")
                .required(true),
        );
    let command = command_parser.get_matches();
    let ncodes = command.get_many::<String>("ncodes").unwrap();

    for ncode in ncodes {
        if let Err(x) = make_epub(
            &ncode,
            command.get_flag("horizontal"),
            *command.get_one::<f64>("wait").unwrap(),
        ) {
            println!("{}", x);
            std::process::exit(2);
        }
    }
}
