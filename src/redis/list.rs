use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

/// A list that is stored in Redis.
///
/// # Example
/// ```
/// use dtypes::redis::List;
///
/// let client = redis::Client::open("redis://localhost:6379").unwrap();
/// let mut list = List::new("test_list", client);
/// list.push_back(&1);
/// list.push_back(&2);
/// assert_eq!(list.len(), 2);
/// assert_eq!(list.pop_front(), Some(1));
/// list.clear();
/// ```
pub struct List<T> {
    key: String,
    client: redis::Client,
    _conn: Option<redis::Connection>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> List<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Creates a new List
    ///
    /// There is no `with_value` method like [Generic::with_value] because it is not possible to
    /// provide a good default behaviour in redis. So you have to think about, how you want to handle
    /// already stored values in redis.
    /// If you want a small performance boost, look at [ListCache].
    pub fn new(key: &str, client: redis::Client) -> Self {
        Self {
            client,
            key: key.to_string(),
            _conn: None,
            _phantom: Default::default(),
        }
    }

    /// Returns an iterator over the list.
    pub fn iter(&self) -> ListIter<T> {
        let len = self.len();
        ListIter {
            list: self,
            index: 0,
            len,
        }
    }

    /// Add a value to the front of the list
    pub fn push_front(&mut self, val: &T) {
        let mut conn = self.client.get_connection().unwrap();
        redis::Cmd::lpush(
            &self.key,
            serde_json::to_string(val).expect("Failed to serialize value"),
        )
        .execute(&mut conn);
    }

    /// Add a value to the back of the list
    pub fn push_back(&mut self, val: &T) {
        let mut conn = self.client.get_connection().unwrap();
        redis::Cmd::rpush(
            &self.key,
            serde_json::to_string(val).expect("Failed to serialize value"),
        )
        .execute(&mut conn);
    }

    /// Removes and returns the first value of the list
    pub fn pop_front(&mut self) -> Option<T> {
        let mut conn = self.client.get_connection().unwrap();
        let val: Option<String> = redis::Cmd::lpop(&self.key, None).query(&mut conn).ok();
        val.map(|v| serde_json::from_str(&v).expect("Failed to deserialize value"))
    }

    /// Removes and returns the last value of the list
    pub fn pop_back(&mut self) -> Option<T> {
        let mut conn = self.client.get_connection().unwrap();
        let val: Option<String> = redis::Cmd::rpop(&self.key, None).query(&mut conn).ok();
        val.map(|v| serde_json::from_str(&v).expect("Failed to deserialize value"))
    }

    /// Returns the length of the list
    pub fn len(&self) -> usize {
        let mut conn = self.client.get_connection().unwrap();
        let len: usize = redis::Cmd::llen(&self.key).query(&mut conn).unwrap();
        len
    }

    /// Removes all values from the list
    pub fn clear(&self) {
        let mut conn = self.client.get_connection().unwrap();
        redis::Cmd::del(&self.key).execute(&mut conn);
    }

    /// Returns true if the list contains the value
    pub fn contains(&self, val: &T) -> bool
    where
        T: PartialEq,
    {
        let mut conn = self.client.get_connection().unwrap();
        let val: Option<String> = redis::Cmd::lrange(&self.key, 0, -1)
            .query(&mut conn)
            .ok()
            .and_then(|v: Vec<String>| {
                v.into_iter()
                    .find(|v| serde_json::from_str::<T>(v).unwrap() == *val)
            });
        val.is_some()
    }

    /// Returns true if the list is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// An iterator over the list.
pub struct ListIter<'a, T> {
    list: &'a List<T>,
    index: isize,
    len: usize,
}

impl<'a, T> Iterator for ListIter<'a, T>
where
    T: Serialize + DeserializeOwned,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len as isize {
            return None;
        }

        let mut conn = self.list.client.get_connection().unwrap();
        let val: Option<String> = redis::Cmd::lindex(&self.list.key, self.index)
            .query(&mut conn)
            .ok();
        self.index += 1;
        val.map(|v| serde_json::from_str(&v).expect("Failed to deserialize value"))
    }
}

/// A list that caches the values in memory
/// It improves the performance, if you perform a lot of read only operations on the list.
///
/// All manipulations are done on the cache and synced with the redis server.
///
/// # Example
/// ```
/// use dtypes::redis::{ListCache, Mutex};
///
/// let client = redis::Client::open("redis://localhost:6379").unwrap();
/// let mut list = ListCache::new("test_list2", client);
/// list.push_back(1);
/// list.push_back(2);
/// assert_eq!(list.len(), 2);
/// assert_eq!(list.pop_front(), Some(1));
/// list.clear();
/// ```
pub struct ListCache<T> {
    list: List<T>,
    cache: VecDeque<T>,
}

impl<T> ListCache<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Creates a new ListCache
    /// The list is loaded from the redis server.
    /// If you want to create an empty list, use [ListCache::without_load]
    pub fn new(key: &str, client: redis::Client) -> Self {
        let mut s = Self::without_load(key, client);
        s.pull();
        s
    }

    /// Creates a new ListCache without loading the list from the redis server.
    pub fn without_load(key: &str, client: redis::Client) -> Self {
        let list = List::new(key, client);
        let val = VecDeque::new();
        Self { list, cache: val }
    }

    pub fn pull(&mut self) {
        let mut conn = self.list.client.get_connection().unwrap();
        let val: VecDeque<T> = redis::Cmd::lrange(&self.list.key, 0, -1)
            .query(&mut conn)
            .ok()
            .and_then(|v: Vec<String>| {
                Option::from({
                    v.into_iter()
                        .map(|v| serde_json::from_str::<T>(&v).unwrap())
                        .collect::<VecDeque<T>>()
                })
            })
            .unwrap_or_default();
        self.cache = val
    }

    pub fn push_back(&mut self, val: T) {
        self.list.push_back(&val);
        self.cache.push_back(val);
    }

    pub fn push_front(&mut self, val: T) {
        self.list.push_front(&val);
        self.cache.push_front(val);
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.list.pop_back();
        self.cache.pop_back()
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.list.pop_front();
        self.cache.pop_front()
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn insert(&mut self, index: usize, val: T) {
        self.cache.insert(index, val);
        self.list.push_back(self.cache.get(index).unwrap());
    }

    pub fn front(&self) -> Option<&T> {
        self.cache.front()
    }

    pub fn back(&self) -> Option<&T> {
        self.cache.back()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.cache.get(index)
    }
}

impl<T> Deref for ListCache<T> {
    type Target = List<T>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl<T> DerefMut for ListCache<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}
