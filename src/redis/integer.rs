//! The integer module contains the Ti32 struct which is a wrapper around an i32 value stored in Redis.

use crate::redis::Generic;
pub type Tusize = Generic<usize>;
pub type Tu8 = Generic<u8>;
pub type Tu16 = Generic<u16>;
pub type Tu32 = Generic<u32>;
pub type Tu64 = Generic<u64>;

pub type Tisize = Generic<isize>;

pub type Ti8 = Generic<i8>;
pub type Ti16 = Generic<i16>;
pub type Ti32 = Generic<i32>;
pub type Ti64 = Generic<i64>;

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::assign_op_pattern)]
    #[test]
    fn test_add() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::with_value(1, "test_add", client.clone());
        i32 = i32 + Ti32::with_value(2, "test_add2", client.clone());
        assert_eq!(i32, 3);
    }

    #[test]
    fn test_sub() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::with_value(1, "test_sub", client.clone());
        i32 = i32 - Ti32::with_value(2, "test_sub2", client.clone());
        assert_eq!(i32, -1);
    }

    #[test]
    fn test_mul() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::with_value(1, "test_mul", client.clone());
        i32 = i32 * Ti32::with_value(2, "test_mul2", client.clone());
        assert_eq!(i32, 2);
    }

    #[test]
    fn test_div() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::with_value(1, "test_div", client.clone());
        i32 = i32 / Ti32::with_value(2, "test_div2", client.clone());
        assert_eq!(i32, 0);
    }

    #[test]
    fn test_multiple_calculations() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::with_value(1, "test_multiple_calculations", client.clone());
        i32 = i32 + Ti32::with_value(2, "test_multiple_calculations2", client.clone());
        i32 = i32 - Ti32::with_value(3, "test_multiple_calculations3", client.clone());
        i32 = i32 * Ti32::with_value(4, "test_multiple_calculations4", client.clone());
        i32 = i32 / Ti32::with_value(5, "test_multiple_calculations5", client.clone());
        assert_eq!(i32, 0);
    }

    #[test]
    fn test_add_assign() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut i32 = Ti32::with_value(1, "test_add_assign", client.clone());
        i32 += Ti32::with_value(2, "test_add_assign2", client.clone());
        assert_eq!(i32, 3);
    }
}
