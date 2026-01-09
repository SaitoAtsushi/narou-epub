use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    InvalidNcode,
    FetchFailed,
    InvalidData,
    InvalidImageType,
    EpubBuildFailed,
    IndicatorFailed,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidNcode => write!(f, "おそらく NCODE の形式が正しくありません。"),
            Error::FetchFailed => write!(f, "データの取得に失敗しました。"),
            Error::InvalidData => write!(f, "データの形式が想定したものではありませんでした。"),
            Error::InvalidImageType => write!(f, "想定していない画像タイプです。"),
            Error::EpubBuildFailed => write!(f, "ePub の生成に失敗しました。"),
            Error::IndicatorFailed => write!(f, "進捗表示に失敗しました。"),
        }
    }
}

impl From<minreq::Error> for Error {
    fn from(_: minreq::Error) -> Self {
        Error::FetchFailed
    }
}

impl From<regex::Error> for Error {
    fn from(_: regex::Error) -> Self {
        Error::InvalidData
    }
}

impl From<epub_builder::Error> for Error {
    fn from(_: epub_builder::Error) -> Self {
        Error::EpubBuildFailed
    }
}

impl From<chrono::ParseError> for Error {
    fn from(_: chrono::ParseError) -> Self {
        Error::InvalidData
    }
}

impl From<indicatif::style::TemplateError> for Error {
    fn from(_: indicatif::style::TemplateError) -> Self {
        Error::IndicatorFailed
    }
}

pub type Result<T> = std::result::Result<T, Error>;
