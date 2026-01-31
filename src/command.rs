use std::default::Default;
use std::mem::MaybeUninit;
use windows_sys::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows_sys::Win32::System::Environment::GetCommandLineW;
use windows_sys::Win32::UI::Shell::CommandLineToArgvW;
use windows_sys::w;

pub struct Cmd {
    pub horizontal: bool,
    pub wait: f64,
    pub ncodes: Vec<String>,
}

pub enum Error {
    UnknownOption,
    Help,
    ParseErrorSecond,
    Version,
    FromUtf16Error,
    SystemErrorCode(u32),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownOption => write!(f, "解釈できないオプションです。"),
            Error::Help => write!(f, include_str!("usage.txt")),
            Error::Version => write!(
                f,
                "{} v{}",
                env!("CARGO_BIN_NAME"),
                env!("CARGO_PKG_VERSION")
            ),
            Error::ParseErrorSecond => write!(f, "秒の指定を解釈できませんでした。"),
            Error::SystemErrorCode(n) => write!(f, "ウィンドウズのシステムエラーです。 ({})", n),
            Error::FromUtf16Error => write!(f, "コマンドラインの文字コードの解釈に失敗しました。"),
        }
    }
}

impl From<WIN32_ERROR> for Error {
    fn from(value: WIN32_ERROR) -> Self {
        Self::SystemErrorCode(value)
    }
}

impl From<std::string::FromUtf16Error> for Error {
    fn from(_: std::string::FromUtf16Error) -> Self {
        Self::FromUtf16Error
    }
}

#[derive(Default)]
enum State {
    #[default]
    Start,
    Options,
    Wait,
    Ncodes,
}

struct Argv<'a> {
    argv: &'a [*mut u16],
}

struct Arg {
    arg: *mut u16,
}

impl Arg {
    fn as_slice(&self) -> &[u16] {
        let mut i = 0;
        while unsafe { *self.arg.add(i) } != 0 {
            i += 1;
        }
        unsafe { std::slice::from_raw_parts(self.arg, i) }
    }
}

struct Args<'a> {
    argv: &'a [*mut u16],
    count: usize,
}

impl Iterator for Args<'_> {
    type Item = Arg;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count >= self.argv.len() {
            None
        } else {
            let count = self.count;
            self.count += 1;
            Some(Arg {
                arg: self.argv[count],
            })
        }
    }
}

impl<'a> Argv<'a> {
    fn new() -> Result<Self, WIN32_ERROR> {
        unsafe {
            let mut argc: MaybeUninit<i32> = MaybeUninit::uninit();
            let arg_list = CommandLineToArgvW(GetCommandLineW(), argc.as_mut_ptr());
            if arg_list.is_null() {
                return Err(GetLastError());
            } else {
                Ok(Self {
                    argv: std::slice::from_raw_parts_mut(arg_list, argc.assume_init() as _),
                })
            }
        }
    }

    fn iter(&'a self) -> Args<'a> {
        Args {
            argv: self.argv,
            count: 0,
        }
    }
}

impl PartialEq<*const u16> for Arg {
    fn eq(&self, other: &*const u16) -> bool {
        let mut p = self.arg;
        let mut q = *other;
        while unsafe { *p } != 0 && unsafe { *q } != 0 && unsafe { *p } == unsafe { *q } {
            unsafe {
                p = p.add(1);
                q = q.add(1);
            }
        }
        unsafe { *p == *q }
    }
}

impl Cmd {
    pub fn parse() -> Result<Self, Error> {
        let mut horizontal = false;
        let mut state = Default::default();
        let mut wait = 1.0;
        let mut ncodes: Vec<String> = vec![];
        for i in Argv::new()?.iter() {
            state = match state {
                State::Start => State::Options,
                State::Options => {
                    if i == w!("--horizontal") {
                        horizontal = true;
                        State::Options
                    } else if i == w!("--wait") || i == w!("-w") {
                        State::Wait
                    } else if i == w!("--help") || i == w!("-h") {
                        return Err(Error::Help);
                    } else if i == w!("--version") || i == w!("-V") {
                        return Err(Error::Version);
                    } else if i == w!("--") {
                        State::Ncodes
                    } else if i.as_slice().starts_with(&['-' as u16]) {
                        return Err(Error::UnknownOption);
                    } else {
                        ncodes.push(String::from_utf16(i.as_slice())?);
                        State::Ncodes
                    }
                }
                State::Wait => {
                    wait = String::from_utf16(i.as_slice())?
                        .parse::<f64>()
                        .or(Err(Error::ParseErrorSecond))?;
                    State::Options
                }
                State::Ncodes => {
                    ncodes.push(String::from_utf16(i.as_slice())?);
                    State::Ncodes
                }
            }
        }
        if ncodes.is_empty() {
            return Err(Error::Help);
        }
        Ok(Self {
            horizontal,
            wait,
            ncodes,
        })
    }
}
