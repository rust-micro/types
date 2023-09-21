use crate::redis::Generic;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum LockError {
    #[error("Locking failed")]
    LockFailed,
    #[error("Unlocking failed")]
    UnlockFailed,
    #[error("No connection to Redis available")]
    NoConnection,
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
    local val = redis.call("get", ARGV[1])
    if redis.call("exists", ARGV[1]) or val == false or val == ARGV[3] then
        redis.call("setex", ARGV[1], ARGV[2], ARGV[3])
        return 1
    end
    return 0"#;

/// The drop script.
/// It is used to drop a value in Redis, so that only the instance that locked it can drop it.
/// Takes 2 Arguments:
/// 1. The key of the value to drop,
/// 2. The value to check.
const DROP_SCRIPT: &str = r#"
    local val = redis.call("get", ARGV[1])
    if val == ARGV[2] then
        redis.call("del", ARGV[1])
        return 1
    end
    return 0"#;

/// The RedisMutex struct.
/// It is used to lock a value in Redis, so that only one instance can access it at a time.
/// You have to use RedisGeneric as the data type.
/// It is a wrapper around the data type you want to store like the Mutex in std.
///
/// The lock is released when the guard is dropped or it expires.
/// The default expiration time is 1000ms. If you need more time, use the [Guard::expand()] function.
pub struct Mutex<T> {
    client: redis::Client,
    conn: Option<redis::Connection>,
    data: Generic<T>,
    key: String,
    uuid: Uuid,
}

impl<T> Mutex<T> {
    pub fn new(client: redis::Client, data: Generic<T>) -> Self {
        Self {
            client,
            key: format!("lock_{}", data.key),
            data,
            conn: None,
            uuid: Uuid::new_v4(),
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
                .arg(&self.key)
                .arg(1000)
                .arg(&self.uuid.to_string())
                .invoke::<i8>(&mut conn)
                .expect("Failed to lock. You should not see this!"),
        ) == LockNum::Fail
        {
            println!("waiting for lock");
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

pub struct Guard<'a, T> {
    lock: &'a mut Mutex<T>,
    expanded: bool,
}

impl<'a, T> Guard<'a, T> {
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
        let expand = redis::Cmd::expire(&self.lock.key, 2000);
        expand.execute(conn);
        self.expanded = true;
    }
}

impl<T> Deref for Guard<'_, T>
where
    T: DeserializeOwned + Serialize + Display,
{
    type Target = Generic<T>;

    fn deref(&self) -> &Self::Target {
        // Safety: The very existence of this Guard guarantees that we have exclusive access to the data.
        &self.lock.data
    }
}

impl<T> DerefMut for Guard<'_, T>
where
    T: DeserializeOwned + Serialize + Display,
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
        let key = &self.lock.key;
        let uuid = &self.lock.uuid;
        script
            .arg(key)
            .arg(uuid.to_string())
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
        let i32 = Di32::new("test_add_locking", client.clone());
        let i32_2 = Di32::new("test_add_locking", client.clone());
        let mut lock: Mutex<i32> = Mutex::new(client.clone(), i32);
        let mut lock2: Mutex<i32> = Mutex::new(client, i32_2);

        thread::scope(|s| {
            let t1 = s.spawn(move || {
                let mut guard = lock2.lock().unwrap();
                guard.store(2);
                assert_eq!(*guard, 2);
            });
            {
                let mut guard = lock.lock().unwrap();
                guard.store(1);
                assert_eq!(*guard, 1);
            }
            t1.join().expect("Failed to join thread1");
        });
    }
}
