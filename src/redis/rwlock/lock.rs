use super::RwLockReadGuard;
use super::RwLockWriteGuard;
use crate::redis::rwlock::constants::{READER_LOCK, UUID_SCRIPT, WRITER_LOCK};
use crate::redis::{Generic, LockError};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

/// A Read-Write Lock.
///
/// This lock is similar to the [std::sync::RwLock](https://doc.rust-lang.org/std/sync/struct.RwLock.html).
/// But it is distributed over multiple instances of the same service.
///
/// # Threads
///
/// If you try to get a writer lock in a thread, which already has a reader lock, you will end up in a deadlock.
/// To use the RwLock in threads, you need a scoped thread.
///  
/// # Examples
///
/// ## Linear usage
/// ```
/// use dtypes::redis::RwLock;
/// use dtypes::redis::Di32;
/// use std::thread;
///
/// let client = redis::Client::open("redis://localhost:6379").unwrap();
/// let client2 = client.clone();
/// let mut i32 = Di32::with_value(1, "test_rwlock_example1", client.clone());
/// let mut lock = RwLock::new(i32);
///
/// // many reader locks can be held at once
/// {
///     let read1 = lock.read().unwrap();
///     let read2 = lock.read().unwrap();
///     assert_eq!(*read1, 1);
/// } // read locks are dropped at this point
///
/// // only one writer lock can be held, however
/// {
///     let mut write1 = lock.write().unwrap();
///     write1.store(2);
///     assert_eq!(*write1, 2);
/// } // write lock is dropped here
///
/// // look, you can read it again
/// {
///    let read1 = lock.read().unwrap();
///    assert_eq!(*read1, 2);
/// }
/// ```
/// ## Threaded usage
/// ```
/// use dtypes::redis::RwLock;
/// use dtypes::redis::Di32;
/// use std::thread;
///
/// let client = redis::Client::open("redis://localhost:6379").unwrap();
/// let i32 = Di32::with_value(1, "test_rwlock_example2", client.clone());
/// let mut lock = RwLock::new(i32);
/// // the reader lock is dropped immediately
/// assert_eq!(*lock.read().unwrap(), 1);
/// // Scoped threads are needed, otherwise the lifetime is unclear.
/// thread::scope(|s| {
///        s.spawn(|| {
///            let mut write = lock.write().unwrap();
///            write.store(2);
///            assert_eq!(*write, 2);
///        }).join().unwrap();
/// });
/// assert_eq!(*lock.read().unwrap(), 2);
/// ```
pub struct RwLock<T> {
    pub(crate) data: Generic<T>,
    writer_wanted: bool,
}

impl<T> RwLock<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(data: Generic<T>) -> Self {
        Self {
            data,
            writer_wanted: false,
        }
    }

    /// Creates a new RwLock Reader.
    ///
    /// This function blocks until the lock is acquired.
    /// If there is a writer lock, this function blocks until the writer lock is dropped.
    /// Also if there is a writer locks waiting to be acquired, this function blocks until the writer lock is acquired and dropped.
    pub fn read(&self) -> Result<RwLockReadGuard<T>, LockError> {
        let uuid = self.generate_uuid();
        loop {
            // small performance improvement, because if there is a local writer lock wanted, we don't need to check the remote writer lock
            if self.writer_wanted {
                continue;
            }
            let res = self.execute_script(&uuid, READER_LOCK);
            if res {
                break;
            }
        }
        Ok(RwLockReadGuard::new(self, uuid))
    }

    /// Creates a new RwLock Writer.
    ///
    /// This function blocks until the lock is acquired.
    /// If there is a reader lock, this function blocks until the reader lock is dropped.
    /// The acquiring writer lock has priority over any waiting reader lock.
    pub fn write(&mut self) -> Result<RwLockWriteGuard<T>, LockError> {
        self.writer_wanted = true;
        let uuid = self.generate_uuid();
        loop {
            let res = self.execute_script(&uuid, WRITER_LOCK);
            if res {
                break;
            }
        }
        self.writer_wanted = false;
        Ok(RwLockWriteGuard::new(self, uuid))
    }

    fn execute_script(&self, uuid: &usize, script: &str) -> bool {
        let mut conn = self.data.client.get_connection().unwrap();
        redis::Script::new(script)
            .arg(&self.data.key)
            .arg(uuid)
            .invoke(&mut conn)
            .unwrap()
    }

    pub(crate) fn generate_uuid(&self) -> usize {
        let mut conn = self.data.client.get_connection().unwrap();
        redis::Script::new(UUID_SCRIPT)
            .arg(&self.data.key)
            .invoke(&mut conn)
            .unwrap()
    }
}

impl<T> Deref for RwLock<T> {
    type Target = Generic<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for RwLock<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis::*;
    use std::mem::ManuallyDrop;

    #[test]
    fn test_rwlock() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let i32 = Di32::with_value(1, "test_rwlock", client.clone());
        let mut lock = RwLock::new(i32);
        {
            // multiple reader locks can be held at once
            let read = lock.read().unwrap();
            assert_eq!(*read, 1);
            let read2 = lock.read().unwrap();
            assert_eq!(*read2, 1);
        }
        {
            // only one writer lock can be held, however
            let mut write = lock.write().unwrap();
            write.store(2);
            assert_eq!(*write, 2);
        }
        // look, you can read it again
        let read = lock.read().unwrap();
        assert_eq!(*read, 2);
    }

    #[test]
    fn test_rwlock_deadlock() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let i32 = Di32::with_value(1, "test_rwlock_deadlock", client.clone());
        let mut lock = RwLock::new(i32);
        {
            let _ = ManuallyDrop::new(lock.read().unwrap());
        }
        // This should not deadlocked forever
        // FIXME: This test blocks Pull request, because if a reader lock gets not dropped correctly. The whole systems blocks indefinitely.
        {
            let _ = lock.write().unwrap();
        }

        {
            let _ = ManuallyDrop::new(lock.write().unwrap());
        }
        // This should not deadlocked forever
        // FIXME: This tests the same as above, but for writer locks.
        {
            let _ = lock.read().unwrap();
        }
    }
}
