pub mod episode;
mod error;
mod unescape;
use crate::epub::time::Time;
use episode::EpisodeIter;
pub use error::{Error, Result};
use unescape::Unescape;
pub const AGENT_NAME: &str = "narou-epub-agent/1.0";
use crate::epub::{IdIter, NameId};
use crate::json::{JsonNode, Query};

pub struct Novel {
    ncode: String,
    title: String,
    author_name: String,
    author_yomigana: String,
    last_update: Time,
    story: String,
    series: bool,
    episode: u32,
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
        let response: JsonNode = minreq::get(uri)
            .with_header("User-Agent", AGENT_NAME)
            .send()?
            .error_for_status()?
            .as_str()
            .map_err(|_| Error::InvalidData)?
            .parse()?;
        let allcount = Query::new()
            .get(0)
            .get("allcount")
            .execute(&response)
            .and_then(JsonNode::get_number)
            .ok_or(Error::InvalidData)?;
        if allcount != 1 {
            return Err(Error::InvalidData);
        };
        let object = response.get(1).ok_or(Error::InvalidData)?;
        let title = object
            .get("title")
            .and_then(JsonNode::get_string)
            .ok_or(Error::InvalidData)?
            .unescape();
        let series = match object.get("noveltype") {
            Some(JsonNode::Number(1)) => true,
            Some(JsonNode::Number(2)) => false,
            _ => return Err(Error::InvalidData),
        };
        let userid: u32 = object
            .get("userid")
            .and_then(JsonNode::get_number)
            .ok_or(Error::InvalidData)?;
        let author_name = object
            .get("writer")
            .and_then(JsonNode::get_string)
            .ok_or(Error::InvalidData)?
            .unescape();
        let story = object
            .get("story")
            .and_then(JsonNode::get_string)
            .ok_or(Error::InvalidData)?
            .unescape();
        let last_update: Time = object
            .get("novelupdated_at")
            .and_then(JsonNode::get_string)
            .ok_or(Error::InvalidData)?
            .parse()?;
        let episode = object
            .get("general_all_no")
            .and_then(JsonNode::get_number)
            .ok_or(Error::InvalidData)?;
        let uri = format!("https://api.syosetu.com/userapi/api/?userid={userid}&out=json&of=y",);
        let response: JsonNode = minreq::get(uri)
            .with_header("User-Agent", AGENT_NAME)
            .send()?
            .error_for_status()?
            .as_str()
            .map_err(|_| Error::InvalidData)?
            .parse()?;
        let allcount = Query::new()
            .get(0)
            .get("allcount")
            .execute(&response)
            .and_then(JsonNode::get_number)
            .ok_or(Error::InvalidData)?;
        if allcount != 1 {
            return Err(Error::InvalidData);
        };
        let author_yomigana = Query::new()
            .get(1)
            .get("yomikata")
            .execute(&response)
            .and_then(JsonNode::get_string)
            .ok_or(Error::InvalidData)?;
        Ok(Novel {
            ncode: ncode.to_string(),
            title: title,
            author_name,
            author_yomigana,
            last_update,
            story,
            series,
            episode,
        })
    }

    pub fn episodes(&self) -> Result<EpisodeIter> {
        Ok(EpisodeIter {
            cur: 1,
            max: self.episode,
            series: self.series,
            ncode: self.ncode.clone(),
            id: IdIter::<NameId>::new(),
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

    pub fn last_update(&self) -> &Time {
        &self.last_update
    }

    pub fn episode(&self) -> u32 {
        self.episode
    }
}
