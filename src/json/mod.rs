/// JSON パーサだが数値は符号無し整数のみをサポート
mod lexer;
mod parser;
mod query;
pub use parser::{Error, JsonNode};
pub use query::Query;
