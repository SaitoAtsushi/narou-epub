pub use super::parser::JsonNode;

enum QueryItem {
    Index(usize),
    Key(String),
}

pub struct Query {
    items: Vec<QueryItem>,
}

mod private {
    pub trait Sealed {}
}

pub trait QueryKey: private::Sealed {
    fn get(&self, queries: &mut Query);
}

impl private::Sealed for usize {}
impl private::Sealed for &str {}

impl QueryKey for usize {
    fn get(&self, queries: &mut Query) {
        queries.items.push(QueryItem::Index(*self));
    }
}

impl QueryKey for &str {
    fn get(&self, queries: &mut Query) {
        queries.items.push(QueryItem::Key(self.to_string()));
    }
}

impl Query {
    pub fn new() -> Self {
        Self { items: vec![] }
    }
    pub fn get<T: QueryKey>(mut self, key: T) -> Self {
        key.get(&mut self);
        self
    }

    pub fn execute<'a>(&self, json: &'a JsonNode) -> Option<&'a JsonNode> {
        let mut j = json;
        for i in &self.items {
            j = match *i {
                QueryItem::Index(n) => j.get(n)?,
                QueryItem::Key(ref k) => j.get(k.as_str())?,
            }
        }
        Some(j)
    }
}

#[cfg(test)]
mod tests {
    use super::Query;

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

        let query1 = Query::new().get(0).get("allcount");
        let query2 = Query::new().get(1).get("title");
        let query3 = Query::new().get(1).get("noveltype");
        let query4 = Query::new().get(1).get("general_all_no");
        let query5 = Query::new().get(1).get("novelupdated_at");
        assert_eq!(query1.execute(&parsed_json), Some(&1.into()));
        assert_eq!(
            query2.execute(&parsed_json),
            Some(&"テスト用タイトル".into())
        );
        assert_eq!(query3.execute(&parsed_json), Some(&1.into()));
        assert_eq!(query4.execute(&parsed_json), Some(&18.into()));
        assert_eq!(
            query5.execute(&parsed_json),
            Some(&"1981-03-08 06:25:17".into())
        );
    }
}
