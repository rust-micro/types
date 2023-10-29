use crate::redis::Generic;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::{Deref, DerefMut};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LockError {
    #[error("Locking failed")]
    LockFailed,
    #[error("Unlocking failed")]
    UnlockFailed,
    #[error("No connection to Redis available")]
    NoConnection,
    #[error("Lock expired with id #{0}")]
    LockExpired(usize),
    #[error("Error by Redis")]
    Redis(#[from] redis::RedisError),
}

#[derive(Debug, PartialEq)]
enum LockNum {
    Success,
    Fail,
}

impl From<i8> for LockNum {
    fn from(value: i8) -> Self {
        match value {
            0 => Self::Fail,
            1 => Self::Success,
            _ => panic!("Unexpected value"),
        }
    }
}

/// The lock script.
/// It is used to lock a value in Redis, so that only one instance can access it at a time.
/// Takes 3 Arguments:
/// 1. The key of the value to lock,
/// 2. The timeout in seconds,
/// 3. The value to store.
const LOCK_SCRIPT: &str = r#"
local val = redis.call("get", ARGV[1] .. ":lock")
if val == false or val == ARGV[3] then
    redis.call("setex", ARGV[1] .. ":lock", ARGV[2], ARGV[3])
    return 1
end
return 0"#;

/// The drop script.
/// It is used to drop a value in Redis, so that only the instance that locked it can drop it.
///
/// Takes 2 Arguments:
/// 1. The key of the value to drop,
/// 2. The value to check.
const DROP_SCRIPT: &str = r#"
local current_lock = redis.call("get", ARGV[1] .. ":lock")
if current_lock == ARGV[2] then
    redis.call("del", ARGV[1] .. ":lock")
    return 1
end
return 0"#;

/// The uuid script.
/// It is used to generate a uuid for the lock.
/// It is a very simple counter that is stored in Redis and returns all numbers only once.
///
/// Takes 1 Argument:
/// 1. The key of the value to lock.
const UUID_SCRIPT: &str = r#"
redis.call("incr", ARGV[1] .. ":uuids")
local val = redis.call("get", ARGV[1] .. ":uuids")
return val"#;

/// The store script.
/// It is used to store a value in Redis with a lock.
///
/// Takes 3 Arguments:
/// 1. The key of the value to store,
/// 2. The uuid of the lock object,
/// 3. The value to store.
const STORE_SCRIPT: &str = r#"
local current_lock = redis.call("get", ARGV[1] .. ":lock")
if current_lock == ARGV[2] then
    redis.call("set", ARGV[1], ARGV[3])
    return 1
end
return 0"#;

/// The load script.
/// It is used to load a value from Redis with a lock.
///
/// Takes 2 Arguments:
/// 1. The key of the value to load,
/// 2. The uuid of the lock.
const LOAD_SCRIPT: &str = r#"
local current_lock = redis.call("get", ARGV[1] .. ":lock")
if current_lock == ARGV[2] then
    local val = redis.call("get", ARGV[1])
    return val
end
return nil"#;

/// The RedisMutex struct.
///
/// It is used to lock a value in Redis, so that only one instance can access it at a time.
/// You have to use RedisGeneric as the data type.
/// It is a wrapper around the data type you want to store like the Mutex in std.
///
/// The lock is released when the guard is dropped or it expires.
/// The default expiration time is 1000ms. If you need more time, use the [Guard::expand()] function.
pub struct Mutex<T> {
    conn: Option<redis::Connection>,
    data: Generic<T>,
    uuid: usize,
}

impl<T> Mutex<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(data: Generic<T>) -> Self {
        let mut conn = data
            .client
            .get_connection()
            .expect("Failed to get connection to Redis");

        let uuid = redis::Script::new(UUID_SCRIPT)
            .arg(&data.key)
            .invoke::<usize>(&mut conn)
            .expect("Failed to get uuid");

        Self {
            data,
            conn: Some(conn),
            uuid,
        }
    }

    /// Locks the value in Redis.
    /// This function blocks until the lock is acquired.
    /// It returns a guard that can be used to access the value.
    /// The guard will unlock the value when it is dropped.
    ///
    /// Beware that the value is not locked in the Rust sense and can be set by other instances,
    /// if they skip the locking process and its LOCK_SCRIPT.
    ///
    /// If you try to lock a value that is already locked by another instance in the same scope,
    /// this function will block until the lock is released, which will be happen after the lock
    /// expires (1000ms).
    /// If you need to extend this time, you can use the [Guard::expand()] function.
    ///
    /// # Example
    /// ```
    /// use dtypes::redis::Di32 as i32;
    /// use dtypes::redis::Mutex;
    /// use std::thread::scope;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let client2 = client.clone();
    ///
    /// scope(|s| {
    ///    let t1 = s.spawn(move || {
    ///         let mut i32 = i32::new("test_add_example1", client2);
    ///         let mut lock = Mutex::new(i32);
    ///         let mut guard = lock.lock().unwrap();
    ///         guard.store(2).expect("TODO: panic message");
    ///         assert_eq!(*guard, 2);
    ///     });
    ///     {  
    ///         let mut i32 = i32::new("test_add_example1", client);
    ///         let mut lock = Mutex::new(i32);
    ///         let mut guard = lock.lock().unwrap();
    ///         guard.store(1).expect("Failed to store value");
    ///         assert_eq!(*guard, 1);
    ///     }
    ///     t1.join().expect("Failed to join thread1");
    /// });
    /// ```
    ///
    /// It does not allow any deadlocks, because the lock will automatically release after some time.
    /// So you have to check for errors, if you want to handle them.
    ///
    /// Beware: Your CPU can anytime switch to another thread, so you have to check for errors!
    /// But if you are brave enough, you can drop the result and hope for the best.
    ///
    /// # Example
    /// ```
    /// use std::thread::sleep;
    /// use dtypes::redis::Di32 as i32;
    /// use dtypes::redis::Mutex;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let mut i32 = i32::new("test_add_example2", client.clone());
    /// i32.store(1);
    /// assert_eq!(i32.acquire(), &1);
    /// let mut lock = Mutex::new(i32);
    ///
    /// let mut guard = lock.lock().unwrap();
    /// sleep(std::time::Duration::from_millis(1500));
    /// let res = guard.store(3);
    /// assert!(res.is_err(), "{:?}", res);
    /// ```
    pub fn lock(&mut self) -> Result<Guard<T>, LockError> {
        let mut conn = match self.conn.take() {
            Some(conn) => conn,
            None => self
                .client
                .get_connection()
                .map_err(|_| LockError::LockFailed)?,
        };

        let lock_cmd = redis::Script::new(LOCK_SCRIPT);

        while LockNum::from(
            lock_cmd
                .arg(&self.data.key)
                .arg(1)
                .arg(&self.uuid.to_string())
                .invoke::<i8>(&mut conn)
                .expect("Failed to lock. You should not see this!"),
        ) == LockNum::Fail
        {
            std::hint::spin_loop();
        }

        // store the connection for later use
        self.conn = Some(conn);
        let lock = Guard::new(self)?;

        Ok(lock)
    }
}

impl<T> DerefMut for Mutex<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> Deref for Mutex<T> {
    type Target = Generic<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

/// The guard struct for the Mutex.
/// It is used to access the value and not for you to initialize it by your own.
pub struct Guard<'a, T> {
    lock: &'a mut Mutex<T>,
    expanded: bool,
}

impl<'a, T> Guard<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    fn new(lock: &'a mut Mutex<T>) -> Result<Self, LockError> {
        Ok(Self {
            lock,
            expanded: false,
        })
    }

    /// Expands the lock time by 2000ms from the point on its called.
    /// This is useful if you need to access the value for a longer time.
    ///
    /// But use it with caution, because it can lead to deadlocks.
    /// To avoid deadlocks, we only allow one extension per lock.
    pub fn expand(&mut self) {
        if self.expanded {
            return;
        }

        let conn = self.lock.conn.as_mut().expect("Connection should be there");
        let expand = redis::Cmd::expire(format!("{}:lock", &self.lock.data.key), 2);
        expand.execute(conn);
        self.expanded = true;
    }

    /// Stores the value in Redis.
    /// This function blocks until the value is stored.
    /// Disables the store operation of the guarded value.
    pub fn store(&mut self, value: T) -> Result<(), LockError>
    where
        T: Serialize,
    {
        let conn = self.lock.conn.as_mut().ok_or(LockError::NoConnection)?;
        let script = redis::Script::new(STORE_SCRIPT);
        let result: i8 = script
            .arg(&self.lock.data.key)
            .arg(self.lock.uuid)
            .arg(serde_json::to_string(&value).expect("Failed to serialize value"))
            .invoke(conn)
            .expect("Failed to store value. You should not see this!");
        if result == 0 {
            return Err(LockError::LockExpired(self.lock.uuid));
        }
        self.lock.data.cache = Some(value);
        Ok(())
    }

    /// Loads the value from Redis.
    /// This function blocks until the value is loaded.
    /// Shadows the load operation of the guarded value.
    pub fn acquire(&mut self) -> &T {
        self.lock.data.cache = self.try_get();
        self.lock.data.cache.as_ref().unwrap()
    }

    fn try_get(&mut self) -> Option<T> {
        let conn = self
            .lock
            .conn
            .as_mut()
            .ok_or(LockError::NoConnection)
            .expect("Connection should be there");
        let script = redis::Script::new(LOAD_SCRIPT);
        let result: Option<String> = script
            .arg(&self.lock.data.key)
            .arg(self.lock.uuid)
            .invoke(conn)
            .expect("Failed to load value. You should not see this!");
        let result = result?;

        if result == "nil" {
            return None;
        }
        Some(serde_json::from_str(&result).expect("Failed to deserialize value"))
    }
}

impl<T> Deref for Guard<'_, T>
where
    T: DeserializeOwned + Serialize,
{
    type Target = Generic<T>;

    fn deref(&self) -> &Self::Target {
        // Safety: The very existence of this Guard guarantees that we have exclusive access to the data.
        &self.lock.data
    }
}

impl<T> DerefMut for Guard<'_, T>
where
    T: DeserializeOwned + Serialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: The very existence of this Guard guarantees that we have exclusive access to the data.
        &mut self.lock.data
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        let conn = self.lock.conn.as_mut().expect("Connection should be there");
        let script = redis::Script::new(DROP_SCRIPT);
        script
            .arg(&self.lock.data.key)
            .arg(self.lock.uuid)
            .invoke::<()>(conn)
            .expect("Failed to drop lock. You should not see this!");
    }
}

#[cfg(test)]
mod tests {
    use super::Mutex;
    use crate::redis::Di32;
    use std::thread;
    #[test]
    fn test_create_lock() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let client2 = client.clone();

        thread::scope(|s| {
            let t1 = s.spawn(move || {
                let i32_2 = Di32::new("test_add_locking", client2.clone());
                let mut lock2: Mutex<i32> = Mutex::new(i32_2);
                let mut guard = lock2.lock().unwrap();
                guard.store(2).expect("TODO: panic message");
                assert_eq!(*guard, 2);
            });
            {
                let i32 = Di32::new("test_add_locking", client.clone());
                let mut lock: Mutex<i32> = Mutex::new(i32);
                let mut guard = lock.lock().unwrap();
                guard.store(1).expect("TODO: panic message");
                assert_eq!(*guard, 1);
            }
            t1.join().expect("Failed to join thread1");
        });
    }
}
