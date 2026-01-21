use std::fmt::Display;
use super::internet;
// use windows_sys::Win32::Foundation::WIN32_ERROR;

#[derive(Debug)]
pub enum Error {
    InvalidNcode,
    FetchFailed,
    InvalidData,
    InvalidImageType,
    EpubBuildFailed,
    Interrupted,
    OverWriteFail,
    SystemError(u32),
    IOError,
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
            Error::OverWriteFail => write!(f, "既存の同名ファイルに上書き出来ませんでした。"),
            Error::SystemError(n) => write!(f, "ウィンドウズのシステムエラーです。 ({})", n),
            Error::IOError => write!(f, "データの読み書きに失敗しました。"),
        }
    }
}

impl From<internet::Error> for Error {
    fn from(value: internet::Error) -> Self {
        match value {
            internet::Error::SystemError(n) => Self::SystemError(n),
            _ => Error::FetchFailed,
        }
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

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Error::IOError
    }
}

pub type Result<T> = std::result::Result<T, Error>;
