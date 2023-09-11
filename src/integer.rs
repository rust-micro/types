use std::ops;

use crate::traits::BackedType;
use redis::{Commands, RedisResult};

pub struct Ti32 {
    value: i32,
    field_name: String,
    pub(crate) client: redis::Client,
    pub(crate) conn: Option<redis::Connection>,
}

impl Ti32 {
    pub fn new(value: i32, client: redis::Client) -> Ti32 {
        Ti32 {
            value,
            client,
            conn: None,
            field_name: uuid::Uuid::new_v4().to_string(),
        }
    }
}

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

    #[test]
    fn test_add() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone());
        i32 = i32 + Ti32::new(2, client);
        assert_eq!(i32.value, 3);
    }

    #[test]
    fn test_sub() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone());
        i32 = i32 - Ti32::new(2, client);
        assert_eq!(i32.value, -1);
    }

    #[test]
    fn test_mul() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone());
        i32 = i32 * Ti32::new(2, client);
        assert_eq!(i32.value, 2);
    }

    #[test]
    fn test_div() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone());
        i32 = i32 / Ti32::new(2, client);
        assert_eq!(i32.value, 0);
    }

    #[test]
    fn test_multiple_calculations() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone());
        i32 = i32 + Ti32::new(2, client.clone());
        i32 = i32 - Ti32::new(3, client.clone());
        i32 = i32 * Ti32::new(4, client.clone());
        i32 = i32 / Ti32::new(5, client.clone());
        assert_eq!(i32.value, 0);
    }

    #[test]
    fn test_add_assign() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::new(1, client.clone());
        i32 += Ti32::new(2, client);
        assert_eq!(i32.value, 3);
    }

    #[test]
    fn test_into() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let i32 = Ti32::new(1, client.clone());
        let i: i32 = i32.into();
        assert_eq!(i, 1);
    }

    #[test]
    fn test_from() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let i32 = Ti32::new(1, client.clone());
        let i: i32 = i32.into();
        assert_eq!(i, 1);
    }
}
