use super::lock::RwLock;
use crate::redis::rwlock::constants::{LOAD_SCRIPT, READER_LOCK_DROP};
use crate::redis::Generic;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::Deref;

pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
    uuid: usize,
    conn: redis::Connection,
    cache: Option<T>,
}

impl<'a, T> RwLockReadGuard<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub(crate) fn new(lock: &'a RwLock<T>, uuid: usize, conn: redis::Connection) -> Self {
        Self {
            lock,
            uuid,
            conn,
            cache: None,
        }
    }

    /// Loads the value from Redis.
    /// This function blocks until the value is loaded.
    /// Shadows the load operation of the guarded value.
    pub fn acquire(&mut self) -> &T {
        self.cache = self.try_get();
        self.cache.as_ref().unwrap()
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

impl<'a, T> Deref for RwLockReadGuard<'a, T> {
    type Target = Generic<T>;

    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}

impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        let mut conn = self.client.get_connection().unwrap();
        let _: () = redis::Script::new(READER_LOCK_DROP)
            .arg(&self.lock.data.key)
            .arg(self.uuid)
            .invoke(&mut conn)
            .unwrap();
    }
}
