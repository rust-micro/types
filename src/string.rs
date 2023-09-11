//! # String Type
//! This module contains the string type.
use crate::BackedType;
use crate::RedisGeneric;
use redis::Commands;
use std::ops::{Add, AddAssign};

pub type TString = RedisGeneric<String>;

impl PartialEq<&str> for TString {
    fn eq(&self, other: &&str) -> bool {
        self.value == *other
    }
}

impl Add<&TString> for TString {
    type Output = TString;

    fn add(mut self, rhs: &TString) -> Self::Output {
        let value = std::mem::take(&mut self.value) + &rhs.value;
        self.set(value)
    }
}

impl AddAssign<&str> for TString {
    fn add_assign(&mut self, rhs: &str) {
        let value = std::mem::take(&mut self.value) + rhs;
        let mut conn = self.get_conn();
        let res: redis::RedisResult<()> = conn.set(&self.field_name, &value);
        res.expect("Failed to set value");
        self.value = value;
    }
}

impl AddAssign<&TString> for TString {
    fn add_assign(&mut self, rhs: &TString) {
        let value = std::mem::take(&mut self.value) + &rhs.value;
        let mut conn = self.get_conn();
        let res: redis::RedisResult<()> = conn.set(&self.field_name, &value);
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
        let mut s1 = TString::new("Hello".to_string(), client.clone(), "s1".to_string());
        let mut s2 = TString::new("World".to_string(), client.clone(), "s2".to_string());
        let s3 = TString::new("Together".to_string(), client.clone(), "s3".to_string());
        assert_eq!(s1.value, "Hello");
        assert_eq!(s2.value, "World");
        s1 = s1 + &s2;
        assert_eq!(s1.value, "HelloWorld");
        s2 = s1 + &s3;
        assert_eq!(s2.value, "HelloWorldTogether");
    }

    #[test]
    fn test_partialeq() {
        let client = redis::Client::open("redis://localhost/").unwrap();
        let s1 = TString::new("Hello".to_string(), client.clone(), "s1".to_string());
        assert_eq!(s1, "Hello");
        assert_ne!(s1, "World");
    }
}
