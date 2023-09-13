//! # Boolean Type
//! This module contains the boolean type.
pub type TBool = crate::RedisGeneric<bool>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool() {
        let client = redis::Client::open("redis://localhost/").unwrap();
        let mut b1 = TBool::with_value(true, "b1", client.clone());
        let mut b2 = TBool::with_value(false, "b2", client.clone());
        let b3 = TBool::with_value(true, "b3", client.clone());
        let b4 = TBool::with_value(false, "b4", client.clone());
        assert!(b1.cache.unwrap());
        assert_eq!(b2.cache.unwrap(), false);
        b1.store(false);
        assert!(!(b1.cache.unwrap()));
        b1 = b1 & b3;
        assert!(!(b1.cache.unwrap()));
        b2 = b2 | b4;
        assert_eq!(b2.cache.unwrap(), false);
        b1 = b1 ^ b2;
        assert!(!(b1.cache.unwrap()));
    }

    #[test]
    fn test_equalsign() {
        let client = redis::Client::open("redis://localhost/").unwrap();
        let mut b1 = TBool::with_value(true, "b1", client.clone());
        let b2 = TBool::with_value(false, "b2", client.clone());
        b1.store(false);
        assert!(!b1.cache.unwrap());
        assert!(!b2.cached().unwrap());
    }

    #[test]
    fn test_partialeq() {
        let client = redis::Client::open("redis://localhost/").unwrap();
        let b1 = TBool::with_value(true, "b5", client.clone());
        assert_eq!(b1, true);
        assert_ne!(b1, false);
    }
}
