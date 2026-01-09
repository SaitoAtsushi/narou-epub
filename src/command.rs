use std::default::Default;

pub struct Cmd {
    pub horizontal: bool,
    pub wait: f64,
    pub ncodes: Vec<String>,
}

pub enum Error {
    UnknownOption,
    Help,
    ParseErrorSecond,
    Version(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownOption => write!(f, "解釈できないオプションです。"),
            Error::Help => write!(f, include_str!("usage.txt")),
            Error::Version(name) => write!(f, "{} v{}", name, env!("CARGO_PKG_VERSION")),
            Error::ParseErrorSecond => write!(f, "秒の指定を解釈できませんでした。"),
        }
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

impl Cmd {
    pub fn parse() -> Result<Self, Error> {
        let mut horizontal = false;
        let mut command_name = None;
        let mut state = Default::default();
        let mut wait = 1.0;
        let mut ncodes: Vec<String> = vec![];
        for i in std::env::args() {
            state = match state {
                State::Start => {
                    command_name = Some(i);
                    State::Options
                }
                State::Options => match &i[..] {
                    "--horizontal" => {
                        horizontal = true;
                        State::Options
                    }
                    "--wait" | "-w" => State::Wait,
                    "--help" | "-h" => {
                        return Err(Error::Help);
                    }
                    "--version" | "-V" => {
                        return Err(Error::Version(command_name.unwrap()));
                    }
                    "--" => State::Ncodes,
                    s if s.starts_with('-') => {
                        return Err(Error::UnknownOption);
                    }
                    _ => {
                        ncodes.push(i);
                        State::Ncodes
                    }
                },
                State::Wait => {
                    wait = i.parse::<f64>().or(Err(Error::ParseErrorSecond))?;
                    State::Options
                }
                State::Ncodes => {
                    ncodes.push(i);
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
