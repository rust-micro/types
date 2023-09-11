use crate::generic::RedisGeneric;
use crate::traits::BackedType;
use redis::{Commands, RedisResult};
use std::ops;
use std::ops::AddAssign;

pub type TString = RedisGeneric<String>;

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

impl PartialEq<&str> for TString {
    fn eq(&self, other: &&str) -> bool {
        self.value == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let client = redis::Client::open("redis://localhost/").unwrap();
        let mut s1 = TString::new("Hello".to_string(), client.clone(), "s1".to_string());
        let mut s2 = TString::new("World".to_string(), client.clone(), "s2".to_string());
        let s3 = TString::new("Together".to_string(), client.clone(), "s3".to_string());
        assert_eq!(s1.value, "Hello");
        assert_eq!(s2.value, "World");
        s1 += s2;
        assert_eq!(s1.value, "HelloWorld");
        s2 = s1 + s3;
        assert_eq!(s2.value, "HelloWorldTogether");
    }

    #[test]
    fn test_partialeq() {
        let client = redis::Client::open("redis://localhost/").unwrap();
        let mut s1 = TString::new("Hello".to_string(), client.clone(), "s1".to_string());
        assert_eq!(s1, "Hello");
        assert_ne!(s1, "World");
    }
}
