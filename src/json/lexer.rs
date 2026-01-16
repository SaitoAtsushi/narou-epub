#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    UnexpectedChar(char),
    UnknownEscapeChar(char),
    InvalidCodePoint(u32),
    EarlyTerminate,
}

fn is_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\n' | '\t' | '\r')
}

#[derive(Debug, PartialEq, Clone)]
pub enum JsonValue {
    String(String),
    Number(u32),
    Null,
    Bool(bool),
}

#[derive(Debug, PartialEq, Clone)]
pub enum JsonToken {
    LeftSquare,
    RightSquare,
    LeftCurly,
    RightCurly,
    Colon,
    Comma,
    Value(JsonValue),
}

impl std::convert::From<JsonValue> for JsonToken {
    fn from(value: JsonValue) -> Self {
        JsonToken::Value(value)
    }
}

pub struct Tokens<'a, T> {
    buffer: Option<char>,
    iter: &'a mut T,
}

impl<'a, T: Iterator<Item = char>> Tokens<'a, T> {
    pub fn new(iter: &'a mut T) -> Self {
        Self { buffer: None, iter }
    }
    fn next_with_buffer(&mut self) -> Option<char> {
        std::mem::take(&mut self.buffer).or_else(|| self.iter.next())
    }
    fn token_head(&mut self) -> Option<char> {
        let mut ch = self.next_with_buffer()?;
        while is_whitespace(ch) {
            ch = self.iter.next()?;
        }
        Some(ch)
    }
    fn unget(&mut self, ch: Option<char>) {
        assert!(self.buffer.is_none());
        self.buffer = ch;
    }
    fn lex(&mut self, first_ch: char) -> Result<JsonToken, Error> {
        match first_ch {
            '{' => Ok(JsonToken::LeftCurly),
            '}' => Ok(JsonToken::RightCurly),
            '[' => Ok(JsonToken::LeftSquare),
            ']' => Ok(JsonToken::RightSquare),
            ':' => Ok(JsonToken::Colon),
            ',' => Ok(JsonToken::Comma),
            '"' => {
                let mut newstr = String::new();
                loop {
                    match self.iter.next().ok_or(Error::EarlyTerminate)? {
                        '"' => return Ok(JsonValue::String(newstr).into()),
                        '\\' => match self.iter.next().ok_or(Error::EarlyTerminate)? {
                            'n' => newstr.push('\n'),
                            't' => newstr.push('\t'),
                            'r' => newstr.push('\r'),
                            '\\' => newstr.push('\\'),
                            '/' => newstr.push('/'),
                            'b' => newstr.push('\u{8}'),
                            'f' => newstr.push('\u{C}'),
                            'u' => {
                                let mut acc: u32 = 0;
                                for _ in 0..4 {
                                    let ch = self.iter.next().ok_or(Error::EarlyTerminate)?;
                                    acc = acc * 16
                                        + ch.to_digit(16).ok_or(Error::UnexpectedChar(ch))?;
                                }
                                newstr
                                    .push(char::from_u32(acc).ok_or(Error::InvalidCodePoint(acc))?);
                            }
                            ch => Err(Error::UnknownEscapeChar(ch))?,
                        },
                        ch => newstr.push(ch),
                    }
                }
            }
            'n' => {
                for i in "ull".chars() {
                    let ch = self.iter.next().ok_or(Error::EarlyTerminate)?;
                    if ch != i {
                        Err(Error::UnexpectedChar(ch))?;
                    }
                }
                Ok(JsonValue::Null.into())
            }
            't' => {
                for i in "rue".chars() {
                    let ch = self.iter.next().ok_or(Error::EarlyTerminate)?;
                    if ch != i {
                        Err(Error::UnexpectedChar(ch))?;
                    }
                }
                Ok(JsonValue::Bool(true).into())
            }
            'f' => {
                for i in "alse".chars() {
                    let ch = self.iter.next().ok_or(Error::EarlyTerminate)?;
                    if ch != i {
                        Err(Error::UnexpectedChar(ch))?;
                    }
                }
                Ok(JsonValue::Bool(false).into())
            }
            '0' => Ok(JsonValue::Number(0).into()),
            ch @ '1'..='9' => {
                let mut acc: u32 = ch.to_digit(10).unwrap();
                loop {
                    let ch = self.iter.next();
                    match ch {
                        None => {
                            self.unget(ch);
                        }
                        Some(ch) if ch.is_ascii_digit() => {
                            acc = acc * 10 + ch.to_digit(10).unwrap();
                        }
                        _ => {
                            self.unget(ch);
                            break;
                        }
                    };
                }
                Ok(JsonValue::Number(acc).into())
            }
            _ => Ok(JsonValue::Null.into()),
        }
    }
}

impl<T: Iterator<Item = char>> Iterator for Tokens<'_, T> {
    type Item = Result<JsonToken, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.token_head()?;
        Some(self.lex(ch))
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, JsonToken, JsonValue, Tokens};
    #[test]
    fn it_works() -> Result<(), Error> {
        let json1 = r#"[{"allcount":1},
                              {"title":"\u30c6\u30b9\u30c8\u7528\u30bf\u30a4\u30c8\u30eb",
                               "noveltype":1,
                               "general_all_no":18,
                               "novelupdated_at":"1981-03-08 06:25:17"
                              }
                             ]"#;
        let tokenized_json1 =
            Tokens::new(&mut json1.chars()).collect::<Result<Vec<JsonToken>, Error>>()?;
        let right1 = vec![
            JsonToken::LeftSquare,
            JsonToken::LeftCurly,
            JsonToken::Value(JsonValue::String("allcount".to_string())),
            JsonToken::Colon,
            JsonToken::Value(JsonValue::Number(1)),
            JsonToken::RightCurly,
            JsonToken::Comma,
            JsonToken::LeftCurly,
            JsonToken::Value(JsonValue::String("title".to_string())),
            JsonToken::Colon,
            JsonToken::Value(JsonValue::String("テスト用タイトル".to_string())),
            JsonToken::Comma,
            JsonToken::Value(JsonValue::String("noveltype".to_string())),
            JsonToken::Colon,
            JsonToken::Value(JsonValue::Number(1)),
            JsonToken::Comma,
            JsonToken::Value(JsonValue::String("general_all_no".to_string())),
            JsonToken::Colon,
            JsonToken::Value(JsonValue::Number(18)),
            JsonToken::Comma,
            JsonToken::Value(JsonValue::String("novelupdated_at".to_string())),
            JsonToken::Colon,
            JsonToken::Value(JsonValue::String("1981-03-08 06:25:17".to_string())),
            JsonToken::RightCurly,
            JsonToken::RightSquare,
        ];
        assert_eq!(tokenized_json1, right1);
        Ok(())
    }
}
