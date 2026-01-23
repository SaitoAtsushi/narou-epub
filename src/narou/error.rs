use super::internet;
use std::fmt::Display;
// use windows_sys::Win32::Foundation::WIN32_ERROR;

#[derive(Debug)]
pub enum Error {
    InvalidNcode,
    InvalidData,
    EpubBuildFailure,
    Interrupted,
    OverWriteFail,
    SystemErrorCode(u32),
    IoFailure,
    UnknownImageType,
    InvalidCharCode,
    BadStatus(u32),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidNcode => write!(f, "おそらく NCODE の形式が正しくありません。"),
            Error::InvalidData => write!(f, "データの形式が想定したものではありませんでした。"),
            Error::EpubBuildFailure => write!(f, "ePub の生成に失敗しました。"),
            Error::Interrupted => write!(f, "処理が中断されました。"),
            Error::OverWriteFail => write!(f, "既存の同名ファイルに上書き出来ませんでした。"),
            Error::SystemErrorCode(n) => write!(f, "ウィンドウズのシステムエラーです。 ({})", n),
            Error::IoFailure => write!(f, "データの読み書きに失敗しました。"),
            Error::UnknownImageType => write!(f, "知らない画像形式に遭遇しました。"),
            Error::InvalidCharCode => write!(f, "文字コードが不正です。"),
            Error::BadStatus(code) => write!(
                f,
                "HTTP レスポンスのステータスコード ({}) が想定外です。",
                code
            ),
        }
    }
}

impl From<internet::Error> for Error {
    fn from(value: internet::Error) -> Self {
        match value {
            internet::Error::SystemErrorCode(n) => Self::SystemErrorCode(n),
            internet::Error::InvalidCharCode => Self::InvalidCharCode,
            internet::Error::BadStatus(code) => Self::BadStatus(code),
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
        Error::EpubBuildFailure
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
        Error::IoFailure
    }
}

pub type Result<T> = std::result::Result<T, Error>;
