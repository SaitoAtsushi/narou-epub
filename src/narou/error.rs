use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    InvalidNcode,
    FetchFailed,
    InvalidData,
    InvalidImageType,
    EpubBuildFailed,
    Interrupted,
    OverWriteFail,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidNcode => write!(f, "おそらく NCODE の形式が正しくありません。"),
            Error::FetchFailed => write!(f, "データの取得に失敗しました。"),
            Error::InvalidData => write!(f, "データの形式が想定したものではありませんでした。"),
            Error::InvalidImageType => write!(f, "想定していない画像タイプです。"),
            Error::EpubBuildFailed => write!(f, "ePub の生成に失敗しました。"),
            Error::Interrupted => write!(f, "処理が中断されました。"),
            Error::OverWriteFail => write!(f, "既存の同名ファイルに上書き出来ませんでした。")
        }
    }
}

impl From<minreq::Error> for Error {
    fn from(_: minreq::Error) -> Self {
        Error::FetchFailed
    }
}

impl From<regex_lite::Error> for Error {
    fn from(_: regex_lite::Error) -> Self {
        Error::InvalidData
    }
}

impl From<zip_builder::Error> for Error {
    fn from(_: zip_builder::Error) -> Self {
        Error::EpubBuildFailed
    }
}

impl From<super::super::epub::time::Error> for Error {
    fn from(_: super::super::epub::time::Error) -> Self {
        Error::InvalidData
    }
}

impl From<super::super::json::Error> for Error {
    fn from(_: super::super::json::Error) -> Self {
        Error::InvalidData
    }
}

pub type Result<T> = std::result::Result<T, Error>;
