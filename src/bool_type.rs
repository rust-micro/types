use crate::traits::BackedType;
use redis::{Commands, RedisResult};
use std::ops;

pub struct TBool {
    value: bool,
    field_name: String,
    pub(crate) client: redis::Client,
    pub(crate) conn: Option<redis::Connection>,
}

impl TBool {
    pub fn new(value: bool, client: redis::Client) -> TBool {
        TBool {
            value,
            client,
            conn: None,
            field_name: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl ops::Not for TBool {
    type Output = TBool;

    fn not(mut self) -> Self::Output {
        let field = self.field_name.clone();
        let value = !self.value;
        let conn = self.get_conn();
        let res: RedisResult<()> = conn.set(field, value);
        res.expect("Failed to set value");
        self.value = value;
        self
    }
}

impl ops::BitAnd<TBool> for TBool {
    type Output = TBool;

    fn bitand(mut self, rhs: TBool) -> Self::Output {
        let field = self.field_name.clone();
        let value = self.value & rhs.value;
        let conn = self.get_conn();
        let res: RedisResult<()> = conn.set(field, value);
        res.expect("Failed to set value");
        self.value = value;
        self
    }
}

impl ops::BitOr<TBool> for TBool {
    type Output = TBool;

    fn bitor(mut self, rhs: TBool) -> Self::Output {
        let field = self.field_name.clone();
        let value = self.value | rhs.value;
        let conn = self.get_conn();
        let res: RedisResult<()> = conn.set(field, value);
        res.expect("Failed to set value");
        self.value = value;
        self
    }
}

impl ops::BitXor<TBool> for TBool {
    type Output = TBool;

    fn bitxor(mut self, rhs: TBool) -> Self::Output {
        let field = self.field_name.clone();
        let value = self.value ^ rhs.value;
        let conn = self.get_conn();
        let res: RedisResult<()> = conn.set(field, value);
        res.expect("Failed to set value");
        self.value = value;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool() {
        let client = redis::Client::open("redis://localhost/").unwrap();
        let mut b1 = TBool::new(true, client.clone());
        let mut b2 = TBool::new(false, client.clone());
        let b3 = TBool::new(true, client.clone());
        let b4 = TBool::new(false, client.clone());
        assert!(b1.value);
        assert!(!b2.value);
        b1 = !b1;
        assert!(!b1.value);
        b1 = b1 & b3;
        assert!(!b1.value);
        b2 = b2 | b4;
        assert!(!b2.value);
        b1 = b1 ^ b2;
        assert!(!b1.value);
    }
}
