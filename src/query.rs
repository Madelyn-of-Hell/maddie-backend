pub trait Query {
    fn query(&self) -> Option<&str>;
}
impl Query for &str {
    fn query(&self) -> Option<&str> {
        self.split("/?").last()
    }
}