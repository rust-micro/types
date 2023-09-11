use crate::traits::BackedType;
use redis::{Commands, RedisResult, ToRedisArgs};
use std::fmt::Debug;
use std::ops;

pub struct RedisGeneric<T> {
    pub(crate) value: T,
    pub(crate) field_name: String,
    client: redis::Client,
}

impl<T> RedisGeneric<T> {
    /// The new method creates a new instance of the type.
    ///
    /// # Example
    ///
    /// ```
    /// use types::i32;
    /// use types::BackedType;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let mut i32 = i32::new(1, client.clone(), "test_add".to_string());
    /// i32 = i32 + i32::new(2, client, "test_add2".to_string());
    /// assert_eq!(i32, 3);
    /// ```
    pub fn new(value: T, client: redis::Client, field_name: String) -> RedisGeneric<T> {
        RedisGeneric {
            value,
            client,
            field_name,
        }
    }

    /// The set method sets the value of the type.
    pub fn set(mut self, value: T) -> Self
    where
        T: ToRedisArgs,
    {
        let mut conn = self.get_conn();
        let res: RedisResult<()> = conn.set(&self.field_name, &value);
        res.expect("Failed to set value");
        self.value = value;
        self
    }

    /// The get method returns a reference to the value stored in the type.
    ///
    /// # Example
    ///
    /// ```
    /// use types::i32;
    /// use types::BackedType;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let mut i32 = i32::new(1, client.clone(), "test_add".to_string());
    /// i32 = i32 + i32::new(2, client, "test_add2".to_string());
    /// assert_eq!(i32.get(), &3);
    /// ```
    pub fn get(&self) -> &T {
        &self.value
    }

    /// The into_inner method returns the inner value of the type.
    /// This method consumes the type.
    ///
    /// # Example
    ///
    /// ```
    /// use types::i32;
    /// use types::BackedType;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let mut i32 = i32::new(1, client.clone(), "test_add".to_string());
    /// i32 = i32 + i32::new(2, client, "test_add2".to_string());
    /// let i32_inner = i32.into_inner();
    /// assert_eq!(i32_inner, 3);
    /// ```
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> BackedType<T> for RedisGeneric<T> {
    fn get_conn(&self) -> redis::Connection {
        let conn = self
            .client
            .get_connection()
            .expect("Failed to get connection");
        conn
    }

    fn get(&self) -> &T {
        &self.value
    }
}

impl<T> ops::Deref for RedisGeneric<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> ops::Add<T> for RedisGeneric<T>
where
    T: ops::Add<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn add(self, rhs: T) -> Self::Output {
        let value = self.value + rhs;
        self.set(value)
    }
}

impl<T> ops::Add<RedisGeneric<T>> for RedisGeneric<T>
where
    T: ops::Add<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn add(self, rhs: RedisGeneric<T>) -> Self::Output {
        let value = self.value + rhs.value;
        self.set(value)
    }
}

impl<T> ops::Sub<T> for RedisGeneric<T>
where
    T: ops::Sub<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn sub(self, rhs: T) -> Self::Output {
        let value = self.value - rhs;
        self.set(value)
    }
}

impl<T> ops::Sub<RedisGeneric<T>> for RedisGeneric<T>
where
    T: ops::Sub<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn sub(self, rhs: RedisGeneric<T>) -> Self::Output {
        let value = self.value - rhs.value;
        self.set(value)
    }
}

impl<T> ops::Mul<T> for RedisGeneric<T>
where
    T: ops::Mul<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn mul(self, rhs: T) -> Self::Output {
        let value = self.value * rhs;
        self.set(value)
    }
}

impl<T> ops::Mul<RedisGeneric<T>> for RedisGeneric<T>
where
    T: ops::Mul<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn mul(self, rhs: RedisGeneric<T>) -> Self::Output {
        let value = self.value * rhs.value;
        self.set(value)
    }
}

impl<T> ops::Div<T> for RedisGeneric<T>
where
    T: ops::Div<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn div(self, rhs: T) -> Self::Output {
        let value = self.value / rhs;
        self.set(value)
    }
}

impl<T> ops::Div<RedisGeneric<T>> for RedisGeneric<T>
where
    T: ops::Div<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn div(self, rhs: RedisGeneric<T>) -> Self::Output {
        let value = self.value / rhs.value;
        self.set(value)
    }
}

impl<T> ops::AddAssign<T> for RedisGeneric<T>
where
    T: ops::AddAssign + ToRedisArgs,
{
    fn add_assign(&mut self, rhs: T) {
        let mut conn = self.get_conn();
        let res: RedisResult<()> = conn.incr(&self.field_name, &rhs);
        res.expect("Failed to set value");
        self.value += rhs;
    }
}

impl<T> ops::AddAssign<RedisGeneric<T>> for RedisGeneric<T>
where
    T: ops::AddAssign + ToRedisArgs,
{
    fn add_assign(&mut self, rhs: RedisGeneric<T>) {
        let mut conn = self.get_conn();
        let res: RedisResult<()> = conn.incr(&self.field_name, &rhs.value);
        res.expect("Failed to set value");
        self.value += rhs.value;
    }
}

impl<T> ops::SubAssign<T> for RedisGeneric<T>
where
    T: ops::SubAssign + ToRedisArgs,
{
    fn sub_assign(&mut self, rhs: T) {
        let mut conn = self.get_conn();
        let res: RedisResult<()> = conn.decr(&self.field_name, &rhs);
        res.expect("Failed to set value");
        self.value -= rhs;
    }
}

impl<T> ops::SubAssign<RedisGeneric<T>> for RedisGeneric<T>
where
    T: ops::SubAssign + ToRedisArgs,
{
    fn sub_assign(&mut self, rhs: RedisGeneric<T>) {
        let mut conn = self.get_conn();
        let res: RedisResult<()> = conn.decr(&self.field_name, &rhs.value);
        res.expect("Failed to set value");
        self.value -= rhs.value;
    }
}

impl<T> ops::BitOr<T> for RedisGeneric<T>
where
    T: ops::BitOr<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn bitor(self, rhs: T) -> Self::Output {
        let value = self.value | rhs;
        self.set(value)
    }
}

impl<T> ops::BitOr<RedisGeneric<T>> for RedisGeneric<T>
where
    T: ops::BitOr<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn bitor(self, rhs: RedisGeneric<T>) -> Self::Output {
        let value = self.value | rhs.value;
        self.set(value)
    }
}

impl<T> ops::BitAnd<T> for RedisGeneric<T>
where
    T: ops::BitAnd<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn bitand(self, rhs: T) -> Self::Output {
        let value = self.value & rhs;
        self.set(value)
    }
}

impl<T> ops::BitAnd<RedisGeneric<T>> for RedisGeneric<T>
where
    T: ops::BitAnd<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn bitand(self, rhs: RedisGeneric<T>) -> Self::Output {
        let value = self.value & rhs.value;
        self.set(value)
    }
}

impl<T> ops::BitXor<T> for RedisGeneric<T>
where
    T: ops::BitXor<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn bitxor(self, rhs: T) -> Self::Output {
        let value = self.value ^ rhs;
        self.set(value)
    }
}

impl<T> ops::BitXor<RedisGeneric<T>> for RedisGeneric<T>
where
    T: ops::BitXor<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn bitxor(self, rhs: RedisGeneric<T>) -> Self::Output {
        let value = self.value ^ rhs.value;
        self.set(value)
    }
}

impl<T> ops::Not for RedisGeneric<T>
where
    T: ops::Not<Output = T> + ToRedisArgs + Copy,
{
    type Output = RedisGeneric<T>;

    fn not(self) -> Self::Output {
        let value = !self.value;
        self.set(value)
    }
}

impl<T: PartialEq> PartialEq<T> for RedisGeneric<T> {
    fn eq(&self, other: &T) -> bool {
        self.value == *other
    }
}

impl<T: PartialEq> PartialEq<RedisGeneric<T>> for RedisGeneric<T> {
    fn eq(&self, other: &RedisGeneric<T>) -> bool {
        self.value == other.value
    }
}

impl<T: Debug> Debug for RedisGeneric<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Generic")
            .field("value", &self.value)
            .field("field_name", &self.field_name)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_partialeq() {
        let s1 = RedisGeneric::new(
            2,
            redis::Client::open("redis://localhost/").unwrap(),
            "s1".to_string(),
        );
        assert_eq!(s1, 2);
    }
}
