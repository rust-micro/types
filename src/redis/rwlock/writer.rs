use crate::redis::rwlock::constants::{LOAD_SCRIPT, STORE_SCRIPT, WRITER_LOCK_DROP};
use crate::redis::rwlock::RwLockError;
use crate::redis::{Generic, RwLock};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

pub struct RwLockWriteGuard<'a, T> {
    lock: &'a mut RwLock<T>,
    conn: redis::Connection,
    uuid: usize,
}

impl<'a, T> RwLockWriteGuard<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub(crate) fn new(lock: &'a mut RwLock<T>, uuid: usize, conn: redis::Connection) -> Self {
        Self { lock, uuid, conn }
    }

    /// Stores the value in Redis.
    /// This function blocks until the value is stored.
    /// Disables the store operation of the guarded value.
    pub fn store(&mut self, value: T) -> Result<(), RwLockError>
    where
        T: Serialize,
    {
        let script = redis::Script::new(STORE_SCRIPT);
        let result: i8 = script
            .arg(&self.lock.data.key)
            .arg(self.uuid)
            .arg(serde_json::to_string(&value).expect("Failed to serialize value"))
            .invoke(&mut self.conn)
            .expect("Failed to store value. You should not see this!");
        if result == 0 {
            return Err(RwLockError::LockExpired(self.uuid));
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
        let script = redis::Script::new(LOAD_SCRIPT);
        let result: Option<String> = script
            .arg(&self.lock.data.key)
            .arg(self.uuid)
            .invoke(&mut self.conn)
            .expect("Failed to load value. You should not see this!");
        let result = result?;

        if result == "nil" {
            return None;
        }
        Some(serde_json::from_str(&result).expect("Failed to deserialize value"))
    }
}

impl<'a, T> Deref for RwLockWriteGuard<'a, T> {
    type Target = Generic<T>;

    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}

impl<'a, T> DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lock.data
    }
}

impl<'a, T> Drop for RwLockWriteGuard<'a, T> {
    fn drop(&mut self) {
        let mut conn = self.client.get_connection().unwrap();
        let _: () = redis::Script::new(WRITER_LOCK_DROP)
            .arg(&self.lock.data.key)
            .arg(self.uuid)
            .invoke(&mut conn)
            .unwrap();
    }
}
