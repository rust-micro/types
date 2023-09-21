//! This module contains the generic type.
//! The generic type is used to implement the common methods for all types.
//! The generic type is not meant to be used directly.
//!
//!
use crate::redis::apply_operator;
use redis::{Commands, RedisResult};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::{Debug, Display};
use std::ops;

pub struct Generic<T> {
    pub(crate) cache: Option<T>,
    pub(crate) key: String,
    client: redis::Client,
}

impl<T> Generic<T>
where
    T: Display + Serialize + DeserializeOwned,
{
    /// The new method creates a new instance of the type.
    /// It does not load or store any value in Redis.
    /// It only creates the instance.
    ///
    /// # Example
    ///
    /// ```
    /// use dtypes::redis::Di32 as i32;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let mut i32 = i32::new("test_add", client.clone());
    /// i32.store(1);
    /// let i32 = i32 + i32::with_value(2, "test_add2", client);
    /// assert_eq!(i32, 3);
    /// ```
    pub fn new(field_name: &str, client: redis::Client) -> Generic<T> {
        Generic {
            cache: None,
            key: field_name.to_string(),
            client,
        }
    }

    /// The with_value method creates a new instance of the type.
    /// If a value is already stored in Redis, it will be overwritten.
    pub fn with_value(value: T, field_name: &str, client: redis::Client) -> Generic<T> {
        let mut new_type = Self::new(field_name, client);

        new_type.store(value);
        new_type
    }

    /// The with_value_load method creates a new instance of the type.
    /// It loads the value from Redis.
    /// If there is no value stored in Redis, it stores a None in cache.
    pub fn with_load(field_name: &str, client: redis::Client) -> Generic<T> {
        let mut new_type = Self::new(field_name, client);

        new_type.cache = new_type.try_get();
        new_type
    }

    /// The with_value_default method creates a new instance of the type.
    /// If the value is not already stored in Redis, it will be stored.
    /// If the value is already stored in Redis, it will be loaded and your given value will be ignored.
    pub fn with_value_default(value: T, field_name: &str, client: redis::Client) -> Generic<T> {
        let mut new_type = Self::new(field_name, client);

        let v = new_type.try_get();
        if v.is_none() {
            new_type.store(value);
        } else {
            new_type.cache = v;
        }

        new_type
    }

    /// The set method sets the value of the type.
    pub fn store(&mut self, value: T) {
        let value = self.set(value);
        self.cache = Some(value);
    }

    /// The set method sets the value of the type in redis.
    /// It does not update the cache.
    /// This is useful if you want to store a value in redis without updating the cache.
    fn set(&self, value: T) -> T {
        let mut conn = self.get_conn();
        let v = serde_json::to_string(&value).expect("Failed to serialize value");
        let res: RedisResult<()> = conn.set(&self.key, v);
        res.expect("Failed to set value");
        value
    }

    /// Pushes the cache to redis.
    fn pushes_to_redis(&self) {
        if self.cache.is_none() {
            return;
        }
        let mut conn = self.get_conn();
        let v = serde_json::to_string(&self.cache).expect("Failed to serialize value");
        let res: RedisResult<()> = conn.set(&self.key, v);
        res.expect("Failed to set value");
    }

    /// The get method returns a reference to the value stored in the type.
    /// Loads it from the redis directly.
    ///
    /// # Example
    ///
    /// ```
    /// use dtypes::redis::Di32 as i32;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let mut i32 = i32::with_value(1, "test_add", client.clone());
    /// i32 = i32 + i32::with_value(2, "test_add2", client);
    /// assert_eq!(i32.acquire(), &3);
    /// ```
    pub fn acquire(&mut self) -> &T {
        self.cache = self.try_get();
        self.cache.as_ref().unwrap()
    }

    pub fn acquire_mut(&mut self) -> &mut T {
        self.cache = self.try_get();
        self.cache.as_mut().unwrap()
    }

    fn try_get(&self) -> Option<T> {
        let mut conn = self.get_conn();
        let res: RedisResult<String> = conn.get(&self.key);
        match res {
            Ok(v) => {
                let v: T = serde_json::from_str(&v).expect("Failed to deserialize value");
                Some(v)
            }
            Err(_) => None,
        }
    }

    /// The into_inner method returns the inner value of the type.
    /// This method consumes the type and drops everything.
    ///
    /// # Example
    ///
    /// ```
    /// use dtypes::redis::Di32 as i32;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let i32 = i32::with_value(3, "test_add", client.clone());
    /// let i32_inner = i32.into_inner();
    /// assert_eq!(i32_inner, 3);
    /// ```
    pub fn into_inner(mut self) -> T {
        let mut conn = self
            .client
            .get_connection()
            .expect("Failed to get connection");
        let _: RedisResult<()> = conn.del(&self.key);
        self.cache.take().expect("Failed to get value")
    }

    /// The get_conn method returns a connection to Redis.
    // FIXME: This should store a persistent connection for performance.
    pub(crate) fn get_conn(&self) -> redis::Connection {
        self.client
            .get_connection()
            .expect("Failed to get connection")
    }

    /// The get method returns a reference to the value stored in the type.
    pub fn cached(&self) -> Option<&T> {
        self.cache.as_ref()
    }
}

impl<T> ops::Deref for Generic<T>
where
    T: Display + Serialize + DeserializeOwned,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.cached().expect("Failed to get value")
    }
}

impl<T> ops::Add<T> for Generic<T>
where
    T: ops::Add<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn add(self, rhs: T) -> Self::Output {
        apply_operator(self, rhs, |a, b| a + b)
    }
}

impl<T> ops::Add<Generic<T>> for Generic<T>
where
    T: ops::Add<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn add(self, rhs: Generic<T>) -> Self::Output {
        self + rhs.into_inner()
    }
}

impl<T> ops::Sub<T> for Generic<T>
where
    T: ops::Sub<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn sub(self, rhs: T) -> Self::Output {
        apply_operator(self, rhs, |a, b| a - b)
    }
}

impl<T> ops::Sub<Generic<T>> for Generic<T>
where
    T: ops::Sub<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn sub(self, rhs: Generic<T>) -> Self::Output {
        self - rhs.into_inner()
    }
}

impl<T> ops::Mul<T> for Generic<T>
where
    T: ops::Mul<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn mul(self, rhs: T) -> Self::Output {
        apply_operator(self, rhs, |a, b| a * b)
    }
}

impl<T> ops::Mul<Generic<T>> for Generic<T>
where
    T: ops::Mul<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn mul(self, rhs: Generic<T>) -> Self::Output {
        self * rhs.into_inner()
    }
}

impl<T> ops::Div<T> for Generic<T>
where
    T: ops::Div<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn div(self, rhs: T) -> Self::Output {
        apply_operator(self, rhs, |a, b| a / b)
    }
}

impl<T> ops::Div<Generic<T>> for Generic<T>
where
    T: ops::Div<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn div(self, rhs: Generic<T>) -> Self::Output {
        self / rhs.into_inner()
    }
}

impl<T> ops::AddAssign<T> for Generic<T>
where
    T: ops::AddAssign + Display + Serialize + DeserializeOwned,
{
    fn add_assign(&mut self, rhs: T) {
        if let Some(ref mut v) = self.cache {
            *v += rhs;
        } else {
            self.cache = Some(rhs);
        }

        self.pushes_to_redis();
    }
}

impl<T> ops::AddAssign<Generic<T>> for Generic<T>
where
    T: ops::AddAssign + Display + Serialize + DeserializeOwned,
{
    fn add_assign(&mut self, rhs: Generic<T>) {
        *self += rhs.into_inner();
    }
}

impl<T> ops::SubAssign<T> for Generic<T>
where
    T: ops::SubAssign + Display + Serialize + DeserializeOwned,
{
    fn sub_assign(&mut self, rhs: T) {
        if let Some(ref mut v) = self.cache {
            *v -= rhs;
        } else {
            self.cache = Some(rhs);
        }

        self.pushes_to_redis();
    }
}

impl<T> ops::SubAssign<Generic<T>> for Generic<T>
where
    T: ops::SubAssign + Display + Serialize + DeserializeOwned,
{
    fn sub_assign(&mut self, rhs: Generic<T>) {
        *self -= rhs.into_inner();
    }
}

impl<T> ops::BitOr<T> for Generic<T>
where
    T: ops::BitOr<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn bitor(self, rhs: T) -> Self::Output {
        apply_operator(self, rhs, |a, b| a | b)
    }
}

impl<T> ops::BitOr<Generic<T>> for Generic<T>
where
    T: ops::BitOr<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn bitor(self, rhs: Generic<T>) -> Self::Output {
        self | rhs.into_inner()
    }
}

impl<T> ops::BitAnd<T> for Generic<T>
where
    T: ops::BitAnd<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn bitand(self, rhs: T) -> Self::Output {
        apply_operator(self, rhs, |a, b| a & b)
    }
}

impl<T> ops::BitAnd<Generic<T>> for Generic<T>
where
    T: ops::BitAnd<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn bitand(self, rhs: Generic<T>) -> Self::Output {
        self & rhs.into_inner()
    }
}

impl<T> ops::BitXor<T> for Generic<T>
where
    T: ops::BitXor<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn bitxor(self, rhs: T) -> Self::Output {
        apply_operator(self, rhs, |a, b| a ^ b)
    }
}

impl<T> ops::BitXor<Generic<T>> for Generic<T>
where
    T: ops::BitXor<Output = T> + Display + Serialize + DeserializeOwned,
{
    type Output = Generic<T>;

    fn bitxor(self, rhs: Generic<T>) -> Self::Output {
        self ^ rhs.into_inner()
    }
}

impl<T: PartialEq> PartialEq<T> for Generic<T> {
    fn eq(&self, other: &T) -> bool {
        self.cache.as_ref() == Some(other)
    }
}

impl<T: PartialEq> PartialEq<Generic<T>> for Generic<T> {
    fn eq(&self, other: &Generic<T>) -> bool {
        self.cache == other.cache
    }
}

impl<T: Debug> Debug for Generic<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Generic")
            .field("value", &self.cache)
            .field("field_name", &self.key)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_partialeq() {
        let s1 = Generic::with_value(
            2,
            "test_partialeq",
            redis::Client::open("redis://localhost/").unwrap(),
        );
        assert_eq!(s1, 2);
    }
}
