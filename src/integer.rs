//! The integer module contains the Ti32 struct which is a wrapper around an i32 value stored in Redis.

pub type Tusize = crate::RedisGeneric<usize>;
pub type Tu8 = crate::RedisGeneric<u8>;
pub type Tu16 = crate::RedisGeneric<u16>;
pub type Tu32 = crate::RedisGeneric<u32>;
pub type Tu64 = crate::RedisGeneric<u64>;

pub type Tisize = crate::RedisGeneric<isize>;

pub type Ti8 = crate::RedisGeneric<i8>;
pub type Ti16 = crate::RedisGeneric<i16>;
pub type Ti32 = crate::RedisGeneric<i32>;
pub type Ti64 = crate::RedisGeneric<i64>;

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
}
