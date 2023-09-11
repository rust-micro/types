//! # Boolean Type
//! This module contains the boolean type.
pub type TBool = crate::RedisGeneric<bool>;

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
