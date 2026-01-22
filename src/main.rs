mod command;
mod epub;
mod indicator;
mod json;
mod narou;
mod sanitize;
mod uuid;
use crate::epub::ReferenceType;
use crate::narou::episode::ImageType;
use epub::{Epub, Escape, IdIter, MediaType, NameId};
use indicator::Indicator;
use narou::episode::ImageInfo;
use regex_lite::Regex;
use sanitize::sanitize;
use std::fs::File;
use std::os::windows::io::{FromRawHandle, OwnedHandle};
use std::sync::atomic::AtomicBool;
use std::thread;
use std::time::Duration;
use windows_sys::Win32::Storage::FileSystem::GetTempFileNameW;
use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
use windows_sys::{
    Win32::{
        Foundation::{GENERIC_WRITE, GetLastError, INVALID_HANDLE_VALUE, MAX_PATH, WIN32_ERROR},
        Storage::FileSystem::{CreateFileW, OPEN_EXISTING},
    },
    w,
};

#[derive(Debug)]
struct TemporaryFile {
    true_name: String,
    temporary_name: String,
    pub handle: Option<File>,
}

impl TemporaryFile {
    pub fn new(true_name: &str) -> Result<Self, WIN32_ERROR> {
        unsafe {
            let mut temporary_name = [0; MAX_PATH as usize];
            if GetTempFileNameW(w!("."), w!("etf"), 0, temporary_name.as_mut_ptr()) == 0 {
                Err(GetLastError())
            } else {
                let handle = CreateFileW(
                    temporary_name.as_ptr(),
                    GENERIC_WRITE,
                    0,
                    std::ptr::null(),
                    OPEN_EXISTING,
                    0,
                    std::ptr::null_mut(),
                );
                if handle == INVALID_HANDLE_VALUE {
                    Err(GetLastError())
                } else {
                    let zero = temporary_name
                        .into_iter()
                        .enumerate()
                        .find(|(_, e)| *e == 0u16)
                        .map(|x| x.0)
                        .unwrap_or(temporary_name.len());
                    let temporary_name = String::from_utf16_lossy(&temporary_name[0..zero]);
                    Ok(Self {
                        temporary_name,
                        true_name: true_name.to_string(),
                        handle: Some(OwnedHandle::from_raw_handle(handle).into()),
                    })
                }
            }
        }
    }

    pub fn finish(&mut self) -> Result<(), narou::Error> {
        if let Some(handle) = std::mem::take(&mut self.handle) {
            drop(handle);
            if std::fs::rename(&self.temporary_name, &self.true_name).is_err() {
                if std::fs::remove_file(&self.temporary_name).is_err() {
                    Err(narou::Error::OverWriteFail)
                } else {
                    Ok(std::fs::rename(&self.temporary_name, &self.true_name)
                        .or(Err(narou::Error::OverWriteFail))?)
                }
            } else {
                Ok(())
            }
        } else {
            panic!();
        }
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        if let Some(handle) = std::mem::take(&mut self.handle) {
            drop(handle);
            let _ = std::fs::remove_file(&self.temporary_name);
        }
    }
}

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
    let valid_pattern = Regex::new("(?i-u)^n[0-9]{4}[[:alpha:]]{0,3}$").unwrap();
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
    eprintln!("{}", novel.title());
    let mut pb = Indicator::new(novel.episode()).ok();
    let mut temporary = TemporaryFile::new(&format!(
        "[{}] {}.epub",
        sanitize(novel.author_name()),
        sanitize(novel.title())
    ))
    .or(Err(narou::Error::EpubBuildFailure))?;
    let mut epub = Epub::new(temporary.handle.as_mut().unwrap())?;
    epub.set_source(format!("https://ncode.syosetu.com/{}/", ncode));
    epub.set_author(
        novel.author_name().to_string(),
        novel.author_yomigana().to_string(),
    );
    // builder.set_uuid(uuid);
    epub.set_title(novel.title().to_string());
    epub.set_modified(novel.last_update().clone());
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
    let mut filename_iter = IdIter::<NameId>::new();
    for i in novel.episodes()? {
        if INTERRUPTED.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(narou::Error::Interrupted);
        }
        if let Some(pb) = pb.as_mut() {
            pb.increment();
        }
        let mut episode = i?;
        // 新しい章の始まり
        if prev_chapter != episode.chapter {
            let chapter_title = episode
                .chapter
                .as_deref()
                .ok_or(narou::Error::InvalidData)?;
            epub.add_content(
                format!("{}.xhtml", filename_iter.next().unwrap()).as_str(),
                chapter_title,
                MediaType::Xhtml,
                1,
                ReferenceType::Text,
                make_chapter(chapter_title).as_bytes(),
            )?;
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
                image_type_to_media_type(image_type),
                ReferenceType::Image,
                &body,
            )?;
        }
        epub.add_content(
            format!("{}.xhtml", filename_iter.next().unwrap()).as_str(),
            &episode.title,
            MediaType::Xhtml,
            if episode.chapter.is_none() { 1 } else { 2 },
            ReferenceType::Text,
            episode.to_string().as_bytes(),
        )?;
        thread::sleep(Duration::from_millis((wait * 1000.0) as u64));
    }
    epub.finish()?;
    drop(epub);
    temporary.finish()?;
    Ok(())
}

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn handler(_: u32) -> i32 {
    INTERRUPTED.store(true, std::sync::atomic::Ordering::SeqCst);
    1
}

fn main() {
    let cmd = match command::Cmd::parse() {
        Err(e) => {
            println!("{}", e);
            std::process::exit(2);
        }
        Ok(s) => s,
    };

    // CTRL+C を押された場合を処理するハンドラを追加
    unsafe { SetConsoleCtrlHandler(Some(handler), 1) };

    for ncode in cmd.ncodes {
        if let Err(x) = make_epub(&ncode, cmd.horizontal, cmd.wait) {
            println!("{}", x);
            std::process::exit(2);
        }
    }
}
