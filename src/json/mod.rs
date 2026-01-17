/// JSON パーサだが数値は符号無し整数のみをサポート
/// サロゲートペアに非対応
mod lexer;
mod parser;
mod query;
pub use parser::{Error, JsonNode};
pub use query::Query;
