pub mod episode;
mod error;
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use episode::EpisodeIter;
pub use error::{Error, Result};
use serde::Deserialize;
use serde_json::{Value, from_value, json};
pub const AGENT_NAME: &str = "narou-epub-agent/1.0";

pub struct Novel {
    ncode: String,
    title: String,
    author_name: String,
    author_yomigana: String,
    last_update: DateTime<Utc>,
    story: String,
    series: bool,
    episode: u32,
}

fn parse_jst_to_utc(s: &str) -> Result<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?;
    let jst = FixedOffset::east_opt(9 * 3600).unwrap();
    let jst_dt = jst.from_local_datetime(&naive).single().unwrap();
    Ok(jst_dt.with_timezone(&Utc))
}

trait StatusCheck {
    fn error_for_status(self) -> Result<Self>
    where
        Self: Sized;
}

impl StatusCheck for minreq::Response {
    fn error_for_status(self) -> Result<Self> {
        if self.status_code == 200 {
            Ok(self)
        } else {
            Err(Error::FetchFailed)
        }
    }
}

impl Novel {
    pub fn new(ncode: &str) -> Result<Self> {
        let uri = format!(
            "https://api.syosetu.com/novelapi/api/?ncode={ncode}&out=json&of=t-nu-s-w-u-nt-ga"
        );
        let response: Value = minreq::get(uri)
            .with_header("User-Agent", AGENT_NAME)
            .send()?
            .error_for_status()?
            .json()?;
        let allcount = response.pointer("/0/allcount").ok_or(Error::InvalidData)?;
        if allcount != &json!(1) {
            return Err(Error::InvalidData);
        };
        let object = response.pointer("/1").ok_or(Error::InvalidData)?;
        #[derive(Deserialize)]
        struct NovelDataApiResult {
            title: String,
            userid: u32,
            writer: String,
            noveltype: u32,
            story: String,
            general_all_no: u32,
            novelupdated_at: String,
        }
        let novel_data: NovelDataApiResult =
            from_value(object.clone()).or(Err(Error::InvalidData))?;
        if novel_data.noveltype != 1 && novel_data.noveltype != 2 {
            return Err(Error::InvalidData);
        };
        let series = novel_data.noveltype == 1;
        let uri = format!(
            "https://api.syosetu.com/userapi/api/?userid={}&out=json&of=y",
            novel_data.userid
        );
        let response: Value = minreq::get(uri)
            .with_header("User-Agent", AGENT_NAME)
            .send()?
            .error_for_status()?
            .json()?;
        let allcount = response.pointer("/0/allcount").ok_or(Error::InvalidData)?;
        if allcount != &json!(1) {
            return Err(Error::InvalidData);
        };
        let object = response.pointer("/1").ok_or(Error::InvalidData)?;
        #[derive(Deserialize)]
        struct UserDataApiResult {
            yomikata: String,
        }
        let user_data: UserDataApiResult =
            from_value(object.clone()).or(Err(Error::InvalidData))?;
        Ok(Novel {
            ncode: ncode.to_string(),
            title: html_escape::decode_html_entities(&novel_data.title).to_string(),
            author_name: html_escape::decode_html_entities(&novel_data.writer).to_string(),
            author_yomigana: user_data.yomikata,
            last_update: parse_jst_to_utc(novel_data.novelupdated_at.as_str())?,
            story: novel_data.story.to_string(),
            series,
            episode: novel_data.general_all_no,
        })
    }

    pub fn episodes(&self) -> Result<EpisodeIter> {
        Ok(EpisodeIter {
            cur: 1,
            max: self.episode,
            series: self.series,
            ncode: self.ncode.clone(),
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn author_name(&self) -> &str {
        &self.author_name
    }

    pub fn author_yomigana(&self) -> &str {
        &self.author_yomigana
    }

    pub fn story(&self) -> &str {
        &self.story
    }

    pub fn last_update(&self) -> DateTime<Utc> {
        self.last_update
    }

    pub fn episode(&self) -> u32 {
        self.episode
    }
}
