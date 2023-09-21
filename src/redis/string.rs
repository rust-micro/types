//! # String Type
//! This module contains the string type.
use crate::redis::Generic;
use std::ops::{Add, AddAssign};

pub type TString = Generic<String>;

impl PartialEq<&str> for TString {
    fn eq(&self, other: &&str) -> bool {
        self.cache.as_ref().map_or(false, |v| v == *other)
    }
}

impl Add<&TString> for TString {
    type Output = TString;

    fn add(mut self, rhs: &TString) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign<&str> for TString {
    fn add_assign(&mut self, rhs: &str) {
        let value = self.cache.take();
        let value = match value {
            Some(mut value) => {
                value.push_str(rhs);
                value
            }
            None => rhs.to_string(),
        };
        self.store(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string() {
        let client = redis::Client::open("redis://localhost/").unwrap();
        let mut s1 = TString::with_value("Hello".to_string(), "s1", client.clone());
        let mut s2 = TString::with_value("World".to_string(), "s2", client.clone());
        let mut s3 = TString::with_value("Together".to_string(), "s3", client.clone());
        assert_eq!(s1, "Hello");
        assert_eq!(s2, "World");
        s1 = s1 + &s2;
        assert_eq!(s1, "HelloWorld");
        s2 = s1 + &s3;
        assert_eq!(s2, "HelloWorldTogether");
        s3 += "test";
        assert_eq!(s3, "Togethertest");
    }

    #[test]
    fn test_partialeq() {
        let client = redis::Client::open("redis://localhost/").unwrap();
        let s1 = TString::with_value("Hello".to_string(), "s1", client.clone());
        assert_eq!(s1, "Hello");
        assert_ne!(s1, "World");
    }
}
