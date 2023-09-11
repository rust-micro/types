use crate::traits::BackedType;
use redis::{Commands, RedisResult};
use std::ops;
use std::ops::AddAssign;

pub struct TString {
    value: String,
    field_name: String,
    pub(crate) conn: Option<redis::Connection>,
    pub(crate) client: redis::Client,
}

impl TString {
    fn new(value: String, client: redis::Client) -> TString {
        TString {
            value,
            client,
            conn: None,
            field_name: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl ops::Add<TString> for TString {
    type Output = TString;

    fn add(mut self, rhs: TString) -> Self::Output {
        let field = self.field_name.clone();
        let value = self.value.clone() + &rhs.value;
        let conn = self.get_conn();
        let res: RedisResult<()> = conn.set(field, &value);
        res.expect("Failed to set value");
        self.value = value;
        self
    }
}

impl AddAssign<TString> for TString {
    fn add_assign(&mut self, rhs: TString) {
        let field = self.field_name.clone();
        let value = self.value.clone() + &rhs.value;
        let conn = self.get_conn();
        let res: RedisResult<()> = conn.set(field, &value);
        res.expect("Failed to set value");
        self.value = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let client = redis::Client::open("redis://localhost/").unwrap();
        let mut s1 = TString::new("Hello".to_string(), client.clone());
        let mut s2 = TString::new("World".to_string(), client.clone());
        let s3 = TString::new("Together".to_string(), client.clone());
        assert_eq!(s1.value, "Hello");
        assert_eq!(s2.value, "World");
        s1 += s2;
        assert_eq!(s1.value, "HelloWorld");
        s2 = s1 + s3;
        assert_eq!(s2.value, "HelloWorldTogether");
    }
}
