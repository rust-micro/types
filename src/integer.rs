//! The integer module contains the Ti32 struct which is a wrapper around an i32 value stored in Redis.
//!
//! # Examples
//!
//! ```
//! use types::i32;
//! use types::BackedType;
//!
//! let client = redis::Client::open("redis://localhost:6379").unwrap();
//! let mut i32 = i32::new(1, client.clone(), "test_add".to_string());
//! i32 = i32 + i32::new(2, client, "test_add2".to_string());
//! assert_eq!(i32, 3);
//! assert_eq!(i32.get(), &3);
//! ```
use std::ops;

use crate::generic::RedisGeneric;
use crate::traits::BackedType;
use redis::{Commands, RedisResult};

pub type Ti32 = RedisGeneric<i32>;

impl ops::Add<Ti32> for Ti32 {
    type Output = Ti32;

    fn add(mut self, rhs: Ti32) -> Self::Output {
        let field = self.field_name.clone();
        let value = self.value + rhs.value;
        let conn = self.get_conn();
        let res: RedisResult<()> = conn.set(field, value);
        res.expect("Failed to set value");
        self.value = value;
        self
    }
}

impl ops::AddAssign<Ti32> for Ti32 {
    fn add_assign(&mut self, rhs: Ti32) {
        let field = self.field_name.clone();
        let value = self.value + rhs.value;
        let conn = self.get_conn();
        let res: RedisResult<()> = conn.set(field, value);
        res.expect("Failed to set value");
        self.value = value;
    }
}

impl ops::Sub<Ti32> for Ti32 {
    type Output = Ti32;

    fn sub(mut self, rhs: Ti32) -> Self::Output {
        let field = self.field_name.clone();
        let value = self.value - rhs.value;
        let conn = self.get_conn();
        let res: RedisResult<()> = conn.set(field, value);
        res.expect("Failed to set value");
        self.value = value;
        self
    }
}

impl ops::Mul<Ti32> for Ti32 {
    type Output = Ti32;

    fn mul(mut self, rhs: Ti32) -> Self::Output {
        let field = self.field_name.clone();
        let value = self.value * rhs.value;
        let conn = self.get_conn();
        let res: RedisResult<()> = conn.set(field, value);
        res.expect("Failed to set value");
        self.value = value;
        self
    }
}

impl ops::Div<Ti32> for Ti32 {
    type Output = Ti32;

    fn div(mut self, rhs: Ti32) -> Self::Output {
        let field = self.field_name.clone();
        let value = self.value / rhs.value;
        let conn = self.get_conn();
        let res: RedisResult<()> = conn.set(field, value);
        res.expect("Failed to set value");
        self.value = value;
        self
    }
}

impl From<Ti32> for i32 {
    fn from(i: Ti32) -> Self {
        i.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::assign_op_pattern)]
    #[test]
    fn test_add() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone(), "test_add".to_string());
        i32 = i32 + Ti32::new(2, client, "test_add2".to_string());
        assert_eq!(i32.value, 3);
    }

    #[test]
    fn test_sub() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone(), "test_sub".to_string());
        i32 = i32 - Ti32::new(2, client, "test_sub2".to_string());
        assert_eq!(i32.value, -1);
    }

    #[test]
    fn test_mul() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone(), "test_mul".to_string());
        i32 = i32 * Ti32::new(2, client, "test_mul2".to_string());
        assert_eq!(i32.value, 2);
    }

    #[test]
    fn test_div() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone(), "test_div".to_string());
        i32 = i32 / Ti32::new(2, client, "test_div2".to_string());
        assert_eq!(i32.value, 0);
    }

    #[test]
    fn test_multiple_calculations() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone(), "test_multiple_calculations".to_string());
        i32 = i32 + Ti32::new(2, client.clone(), "test_multiple_calculations2".to_string());
        i32 = i32 - Ti32::new(3, client.clone(), "test_multiple_calculations3".to_string());
        i32 = i32 * Ti32::new(4, client.clone(), "test_multiple_calculations4".to_string());
        i32 = i32 / Ti32::new(5, client.clone(), "test_multiple_calculations5".to_string());
        assert_eq!(i32.value, 0);
    }

    #[test]
    fn test_add_assign() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone(), "test_add_assign".to_string());
        i32 += Ti32::new(2, client, "test_add_assign2".to_string());
        assert_eq!(i32.value, 3);
    }

    #[test]
    fn test_into() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let i32 = Ti32::new(1, client.clone(), "test_into".to_string());
        let i: i32 = i32.into();
        assert_eq!(i, 1);
    }

    #[test]
    fn test_from() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let i32 = Ti32::new(1, client.clone(), "test_from".to_string());
        let i: i32 = i32.into();
        assert_eq!(i, 1);
    }
}
