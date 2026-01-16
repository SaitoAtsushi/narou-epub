#![allow(dead_code)]
pub use super::lexer::JsonToken;
use super::lexer::{Error as LexerError, JsonValue, Tokens};
use std::convert::From;
use std::ops::Index;
use std::str::FromStr;

#[derive(Debug)]
pub enum Error {
    ImpossibleConversion,
    EarlyTerminate,
    UnexpectedToken(JsonToken),
    TokenizeFailure(LexerError),
}

impl std::convert::From<LexerError> for Error {
    fn from(value: LexerError) -> Self {
        Error::TokenizeFailure(value)
    }
}

type JsonArray = Vec<JsonNode>;

#[derive(Debug, PartialEq)]
pub enum JsonNode {
    String(String),
    Number(u32),
    Null,
    Bool(bool),
    Array(Vec<JsonNode>),
    Object(Vec<(String, JsonNode)>),
}

impl From<JsonValue> for JsonNode {
    fn from(value: JsonValue) -> Self {
        match value {
            JsonValue::Bool(v) => JsonNode::Bool(v),
            JsonValue::Null => JsonNode::Null,
            JsonValue::Number(n) => JsonNode::Number(n),
            JsonValue::String(s) => JsonNode::String(s),
        }
    }
}

impl From<&str> for JsonNode {
    fn from(value: &str) -> Self {
        JsonNode::String(value.to_string())
    }
}

impl From<u32> for JsonNode {
    fn from(value: u32) -> Self {
        JsonNode::Number(value)
    }
}

struct Parser<'a, T> {
    buffer: Option<Result<JsonToken, LexerError>>,
    iter: &'a mut T,
}

impl<'a, T: Iterator<Item = Result<JsonToken, LexerError>>> Parser<'a, T> {
    pub fn new(iter: &'a mut T) -> Self {
        Self { buffer: None, iter }
    }
    fn next_with_buffer(&mut self) -> Option<T::Item> {
        std::mem::take(&mut self.buffer).or_else(|| self.iter.next())
    }
    fn unget(&mut self, item: Option<T::Item>) {
        assert!(self.buffer.is_none());
        self.buffer = item;
    }
}

impl<'a, T: Iterator<Item = Result<JsonToken, LexerError>>> Parser<'a, T> {
    fn json_value_parse(&mut self) -> Result<JsonNode, Error> {
        match self
            .next_with_buffer()
            .ok_or(Error::EarlyTerminate)?
            .map_err(Error::TokenizeFailure)?
        {
            JsonToken::Value(v) => Ok(v.into()),
            JsonToken::LeftSquare => self.json_array_parse(),
            JsonToken::LeftCurly => self.json_object_parse(),
            e => Err(Error::UnexpectedToken(e)),
        }
    }

    fn json_array_parse(&mut self) -> Result<JsonNode, Error> {
        let mut arr = Vec::new();

        let tok = self
            .next_with_buffer()
            .ok_or(Error::EarlyTerminate)?
            .map_err(Error::TokenizeFailure)?;
        if tok == JsonToken::RightSquare {
            return Ok(JsonNode::Array(arr));
        } else {
            self.unget(Some(Ok(tok)));
        }

        loop {
            arr.push(self.json_value_parse()?);
            match self
                .next_with_buffer()
                .ok_or(Error::EarlyTerminate)?
                .map_err(Error::TokenizeFailure)?
            {
                JsonToken::Comma => {}
                JsonToken::RightSquare => break,
                tok => Err(Error::UnexpectedToken(tok))?,
            }
        }
        Ok(JsonNode::Array(arr))
    }

    fn json_object_parse(&mut self) -> Result<JsonNode, Error> {
        let mut obj = Vec::new();

        let tok = self
            .next_with_buffer()
            .ok_or(Error::EarlyTerminate)?
            .map_err(Error::TokenizeFailure)?;
        if tok == JsonToken::RightCurly {
            return Ok(JsonNode::Object(obj));
        } else {
            self.unget(Some(Ok(tok)));
        }

        loop {
            let key = self
                .next_with_buffer()
                .ok_or(Error::EarlyTerminate)?
                .map_err(Error::TokenizeFailure)?;

            if let JsonToken::Value(JsonValue::String(key)) = key {
                let assume_colon = self
                    .next_with_buffer()
                    .ok_or(Error::EarlyTerminate)?
                    .map_err(Error::TokenizeFailure)?;
                if assume_colon != JsonToken::Colon {
                    return Err(Error::UnexpectedToken(assume_colon));
                }
                obj.push((key, self.json_value_parse()?));

                match self
                    .next_with_buffer()
                    .ok_or(Error::EarlyTerminate)?
                    .map_err(Error::TokenizeFailure)?
                {
                    JsonToken::Comma => {}
                    JsonToken::RightCurly => break,
                    tok => return Err(Error::UnexpectedToken(tok)),
                }
            } else {
                return Err(Error::UnexpectedToken(key));
            }
        }
        Ok(JsonNode::Object(obj))
    }
}

impl FromStr for JsonNode {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::new(&mut Tokens::new(&mut s.chars())).json_value_parse()
    }
}

pub trait JsonKey {
    fn get<'a>(self, json: &'a JsonNode) -> Option<&'a JsonNode>;
}

impl JsonKey for usize {
    fn get<'a>(self, json: &'a JsonNode) -> Option<&'a JsonNode> {
        if let JsonNode::Array(arr) = json {
            arr.get(self)
        } else {
            None
        }
    }
}

impl JsonKey for &str {
    fn get<'a>(self, json: &'a JsonNode) -> Option<&'a JsonNode> {
        if let JsonNode::Object(arr) = json {
            arr.iter().find_map(|(k, v)| (k == self).then_some(v))
        } else {
            None
        }
    }
}

impl JsonNode {
    pub fn get<T: JsonKey>(&self, key: T) -> Option<&JsonNode> {
        key.get(self)
    }

    pub fn get_string(&self) -> Option<String> {
        match self {
            &JsonNode::String(ref s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn get_number(&self) -> Option<u32> {
        match self {
            &JsonNode::Number(n) => Some(n),
            _ => None,
        }
    }
}

impl<T: JsonKey> Index<T> for JsonNode {
    type Output = JsonNode;
    fn index(&self, index: T) -> &Self::Output {
        index.get(self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::JsonNode;

    #[test]
    fn it_works() {
        const JSON: &str = r#"[{"allcount":1},
                               {"title":"\u30c6\u30b9\u30c8\u7528\u30bf\u30a4\u30c8\u30eb",
                                "noveltype":1,
                                "general_all_no":18,
                                "novelupdated_at":"1981-03-08 06:25:17"
                               }
                              ]"#;

        let parsed_json: JsonNode = JSON.parse().unwrap();
        let right = JsonNode::Array(vec![
            JsonNode::Object(vec![("allcount".into(), 1.into())]),
            JsonNode::Object(vec![
                ("title".into(), "テスト用タイトル".into()),
                ("noveltype".into(), 1.into()),
                ("general_all_no".into(), 18.into()),
                ("novelupdated_at".into(), "1981-03-08 06:25:17".into()),
            ]),
        ]);
        assert_eq!(parsed_json, right);
        assert_eq!(parsed_json[0]["allcount"], 1.into());
        assert_eq!(parsed_json[1]["title"], "テスト用タイトル".into());
        assert_eq!(parsed_json[1]["noveltype"], 1.into());
        assert_eq!(parsed_json[1]["general_all_no"], 18.into());
        assert_eq!(
            parsed_json[1]["novelupdated_at"],
            "1981-03-08 06:25:17".into()
        );
    }
}
