use super::lock::RwLock;
use crate::redis::rwlock::constants::READER_LOCK_DROP;
use crate::redis::Generic;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::Deref;

pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
    uuid: usize,
}

impl<'a, T> RwLockReadGuard<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub(crate) fn new(lock: &'a RwLock<T>, uuid: usize) -> Self {
        Self { lock, uuid }
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
        let client = self.lock.data.client.clone();
        let mut conn = client.get_connection().unwrap();
        let _: () = redis::Script::new(READER_LOCK_DROP)
            .arg(&self.lock.data.key)
            .arg(self.uuid)
            .invoke(&mut conn)
            .unwrap();
    }
}
