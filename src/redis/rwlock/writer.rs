use crate::redis::rwlock::constants::WRITER_LOCK_DROP;
use crate::redis::{Generic, RwLock};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

pub struct RwLockWriteGuard<'a, T> {
    lock: &'a mut RwLock<T>,
    uuid: usize,
}

impl<'a, T> RwLockWriteGuard<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    pub(crate) fn new(lock: &'a mut RwLock<T>, uuid: usize) -> Self {
        Self { lock, uuid }
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
        // FIXME: We have a deadlock, if the Writer will not dropped properly. Same for the reader!
        let client = self.lock.data.client.clone();
        let mut conn = client.get_connection().unwrap();
        let _: () = redis::Script::new(WRITER_LOCK_DROP)
            .arg(&self.lock.data.key)
            .arg(self.uuid)
            .invoke(&mut conn)
            .unwrap();
    }
}
