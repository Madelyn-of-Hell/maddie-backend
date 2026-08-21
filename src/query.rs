use std::collections::HashMap;
use regex::{Captures, Regex};
use urlencoding::decode;

pub trait Query {
    fn query(&self) -> Option<HashMap<String, String>>;
}
pub trait Path {
    fn path(&self) -> Option<Vec<&str>>;
}
impl Query for &str {
    fn query(&self) -> Option<HashMap<String, String>> {
        parse_query(self.split("/?").last())
    }
}
impl Path for &str {
    fn path(&self) -> Option<Vec<&str>> {
        Some(self.split("/?").next()?.split('/').collect::<Vec<&str>>())
    }
}
fn parse_query(query: Option<&str>) -> Option<HashMap<String, String>> {
    let re = Regex::new(r"([^&]+)=([^&]+)&?").ok()?;

    let mut map:HashMap<String, String> = HashMap::new();
    for (_, [key, value]) in re.captures_iter(query?).map(|c: Captures| c.extract()) {
        let _ = map.insert(key.to_string(), decode(value).ok()?.into_owned());
    }
    Some(map)
}