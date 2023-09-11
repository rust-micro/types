use crate::traits::BackedType;
use crate::RedisGeneric;
use redis::{Commands, RedisResult};
use std::ops;

pub type TBool = RedisGeneric<bool>;

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
        let mut b1 = TBool::new(true, client.clone(), "b1".to_string());
        let mut b2 = TBool::new(false, client.clone(), "b2".to_string());
        let b3 = TBool::new(true, client.clone(), "b3".to_string());
        let b4 = TBool::new(false, client.clone(), "b4".to_string());
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

    #[test]
    fn test_partialeq() {
        let client = redis::Client::open("redis://localhost/").unwrap();
        let mut b1 = TBool::new(true, client.clone(), "b1".to_string());
        assert_eq!(b1, true);
        assert_ne!(b1, false);
    }
}
