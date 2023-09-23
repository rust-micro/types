use crate::redis::Generic;
use serde_json::from_str;
use std::ops::{Deref, DerefMut};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SetLoadError {
    #[error("Ordering number is not greater than current number stored in redis.")]
    OrderError,
}

/// This is the set_load script.
/// It is used to set the value if order is greater than the current order.
/// Returns the current value and the current_ordering number.
///
/// It takes 3 arguments:
/// 1. The key of value to set
/// 2. The order_number of the setting operation
/// 3. The value itself to set
const SET_LOAD_SCRIPT: &str = r#"
local key = ARGV[1]
local order = ARGV[2]
local current_order = redis.call("GET", key .. ":order")
if current_order == false or current_order < order then
    redis.call("SET", key .. ":order", order)
    redis.call("SET", key, ARGV[3])
    current_order = order
end
return {redis.call("GET", key), current_order}
"#;

/// This is the load script.
/// It is used to load the value and the order number of the value.
/// Returns the current value and the current ordering number.
///
/// It takes 1 argument:
/// 1. The key of value to load
const LOAD_SCRIPT: &str = r#"
local key = ARGV[1]
return {redis.call("GET", key), redis.call("GET", key .. ":order")}
"#;

/// The SetLoad type.
///
/// It is used to store a value in redis and load it in sync.
/// It tracks automatically an ordering number to ensure that the value is only stored if the order is greater than the current order, mostly from other instances.
/// The value is only stored if the order is greater than the current order.
///
/// This helps to synchronize the value between multiple instances without any locking mechanism.
/// But can results in more network traffic in benefit of less wait time because of locks.
/// Mostly used in situations, where your value changes rarely but read often.
/// Another use case is, when it is okay for you, that the value could be not the latest or
/// computing a derived value multiple times is acceptable.
#[derive(Debug)]
pub struct SetLoad<T> {
    data: Generic<T>,
    counter: usize,
}

impl<T> SetLoad<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    /// Creates a new SetLoad.
    /// The value is loaded from redis directly.
    pub fn new(data: Generic<T>) -> Self {
        let mut s = Self { data, counter: 0 };
        s.load();
        s
    }

    /// Stores the value in the redis server.
    /// The value is only stored if the ordering_number is greater than the current number.
    /// The order is incremented by one before each store.
    ///
    /// # Example
    /// ```
    /// use dtypes::redis::Generic;
    /// use dtypes::redis::SetLoad;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let mut i32 = Generic::with_value(1, "test_add_setload_example1", client.clone());
    /// let mut setload = SetLoad::new(i32);
    /// setload.store(2).unwrap();
    /// assert_eq!(*setload, 2);
    /// ```
    ///
    /// The store can fail if the order is not greater than the current order.
    /// This happens, if the value was set from another instance before.
    ///
    /// # Example
    /// ```
    /// use std::thread;
    /// use dtypes::redis::Generic;
    /// use dtypes::redis::SetLoad;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let client2 = client.clone();
    ///
    /// thread::scope(|s| {
    ///     let t1 = s.spawn(|| {
    ///         let mut i32: Generic<i32> = Generic::new("test_add_setload_example2", client2);
    ///         let mut setload = SetLoad::new(i32);
    ///         while let Err(_) = setload.store(2) {}
    ///         assert_eq!(*setload, 2);
    ///     });
    ///     let mut i32: Generic<i32> = Generic::new("test_add_setload_example2", client);
    ///     let mut setload = SetLoad::new(i32);
    ///     while let Err(_) = setload.store(3) {}
    ///     assert_eq!(*setload, 3);
    ///     t1.join().unwrap();
    /// });
    /// ```
    pub fn store(&mut self, val: T) -> Result<(), SetLoadError> {
        self.counter += 1;
        let val_json = serde_json::to_string(&val).unwrap();
        let (v, order) = self.store_redis(&val_json);

        if let Some(v) = v {
            if self.counter >= order && v == val_json {
                self.data.cache = Some(val);
                return Ok(());
            }
        }
        Err(SetLoadError::OrderError)
    }

    /// Stores the value in the redis server and blocks until succeeds.
    /// Everything else is equal to [SetLoad::store].
    ///
    /// # Example
    /// ```
    /// use std::thread;
    /// use dtypes::redis::Generic;
    /// use dtypes::redis::SetLoad;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let client2 = client.clone();
    ///
    /// thread::scope(|s| {
    ///     let t1 = s.spawn(|| {
    ///         let mut i32: Generic<i32> = Generic::new("test_add_setload_example3", client2);
    ///         let mut setload = SetLoad::new(i32);
    ///         setload.store_blocking(2).unwrap();
    ///         assert_eq!(*setload, 2);
    ///     });
    ///     let mut i32: Generic<i32> = Generic::new("test_add_setload_example3", client);
    ///     let mut setload = SetLoad::new(i32);
    ///     setload.store_blocking(3).unwrap();
    ///     assert_eq!(*setload, 3);
    ///     t1.join().unwrap();
    /// });
    /// ```
    pub fn store_blocking(&mut self, val: T) -> Result<(), SetLoadError> {
        let val_json = serde_json::to_string(&val).unwrap();
        let mut res = self.store_redis(&val_json);

        while self.counter < res.1 || res.0.is_none() || res.0.unwrap() != val_json {
            self.counter = res.1 + 1;
            res = self.store_redis(&val_json);
        }

        self.data.cache = Some(val);
        Ok(())
    }

    fn store_redis(&self, val: &str) -> (Option<String>, usize) {
        let mut conn = self.data.client.get_connection().unwrap();
        redis::Script::new(SET_LOAD_SCRIPT)
            .arg(&self.data.key)
            .arg(self.counter)
            .arg(&val)
            .invoke(&mut conn)
            .expect("Could not execute script")
    }

    /// Loads the value from the redis server.
    /// This is done automatically on creation.
    /// Mostly used for synchronization. Reset the counter to order from redis or 0.
    pub fn load(&mut self) {
        let mut conn = self.data.client.get_connection().unwrap();
        let res: (Option<String>, Option<usize>) = redis::Script::new(LOAD_SCRIPT)
            .arg(&self.data.key)
            .invoke(&mut conn)
            .expect("Could not execute script");

        match res {
            (Some(v), Some(order)) => {
                self.data.cache = Some(from_str(&v).unwrap());
                self.counter = order;
            }
            (Some(v), None) => {
                self.data.cache = Some(from_str(&v).unwrap());
                self.counter = 0;
            }
            (None, Some(c)) => {
                self.data.cache = None;
                self.counter = c;
            }
            _ => {
                self.data.cache = None;
                self.counter = 0;
            }
        }
    }
}

impl<T> Deref for SetLoad<T> {
    type Target = Generic<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for SetLoad<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_set_load() {
        use crate::redis::Generic;
        use crate::redis::SetLoad;

        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let i32: Generic<i32> = Generic::new("test_add_setload", client.clone());
        let mut setload = SetLoad::new(i32);
        setload.store(2).unwrap();
        assert_eq!(*setload, 2);
    }
}
