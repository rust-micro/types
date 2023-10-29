/// The waiting script.
/// Is is used to indicate, if there is a thread waiting for the barrier.
/// Returns 1 if #num thread waiting >= #num threads that should wait. Otherwise 0.
/// If the thread is the leader, it returns 2.
/// Needs to be used n a loop to update expiration time to signal your wait.
///
/// Takes 4 arguments:
/// 1. The key of the barrier.
/// 2. The id of the barrier itself.
/// 3. The number of threads that should wait for the barrier.
/// 4. The timeout in seconds.
const WAITING_SCRIPT: &str = r#"
redis.call("set", ARGV[1] .. ":waiting:" .. ARGV[2], 1, "EX", ARGV[4])

local leader_id = redis.call("get", ARGV[1] .. ":leader")
if leader_id then
    if leader_id == ARGV[2] then
        return 2
    end
    return 1
end

local count = 0
local cursor = "0"

repeat
    local res = redis.call("scan", cursor, "MATCH", ARGV[1] .. ":waiting:*", "COUNT", ARGV[3] + 1)
    if next(res[2]) ~= nil then
        count = count + #res[2]
    end
    cursor = res[1]
until cursor == "0"

if count < tonumber(ARGV[3]) then
    return 0
end

if not leader_id then
    if redis.call("set", ARGV[1] .. ":leader" , ARGV[2], "EX", ARGV[4], "NX") then
        return 2
    end
end

return 1
"#;

/// The reset script.
/// It is used to reset the barrier, so you can reuse it.
/// Essentially it deletes all keys that are used by the barrier.
///
/// Takes 1 Argument:
/// 1. The key of the value to lock.
/// 2. The uuid of the barrier.
/// 3. The number of threads that should wait for the barrier.
const RESET_SCRIPT: &str = r#"
redis.call("del", ARGV[1] .. ":waiting:" .. ARGV[2])

local count = 0
local cursor = "0"

repeat
    local res = redis.call("scan", cursor, "MATCH", ARGV[1] .. ":waiting:*", "COUNT", ARGV[3] + 1)
    if next(res[2]) ~= nil then
        count = count + #res[2]
    end
    cursor = res[1]
until cursor == "0"

-- if it is the last barrier, delete the leader and uuids key
if count == 0 then
    redis.call("del", ARGV[1] .. ":leader")
    redis.call("del", ARGV[1] .. ":uuids")
end
"#;

/// The uuid script.
/// It is used to generate a uuid for the barrier.
/// It is a very simple counter that is stored in Redis and returns all numbers only once.
///
/// Takes 1 Argument:
/// 1. The key of the value to lock.
const UUID_SCRIPT: &str = r#"
redis.call("incr", ARGV[1] .. ":uuids")
local val = redis.call("get", ARGV[1] .. ":uuids")
return val
"#;

pub struct Barrier {
    uuid: usize,
    num: usize,
    key: String,
    _client: redis::Client,
    conn: Option<redis::Connection>,
}

#[derive(PartialEq)]
enum RedisBarrierStatus {
    Waiting,
    Leader,
    Done,
}

impl From<u8> for RedisBarrierStatus {
    fn from(val: u8) -> Self {
        match val {
            0 => RedisBarrierStatus::Waiting,
            1 => RedisBarrierStatus::Done,
            2 => RedisBarrierStatus::Leader,
            _ => panic!("Invalid RedisBarrierStatus"),
        }
    }
}

/// A `BarrierWaitResult` is returned by [`Barrier::wait()`] when all systems
/// in the [`Barrier`] have rendezvoused.
///
/// # Examples
///
/// ```
/// use dtypes::redis::Barrier;
///
/// let client = redis::Client::open("redis://localhost:6379").unwrap();
/// let mut  barrier = Barrier::new(1, "barrier_doc_test", client);
/// let barrier_wait_result = barrier.wait();
/// ```
pub struct BarrierWaitResult(bool);

impl BarrierWaitResult {
    /// Returns `true` if this thread is the "leader thread" for the call to
    /// [`Barrier::wait()`].
    ///
    /// Only one thread will have `true` returned from their result, all other
    /// threads will have `false` returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use dtypes::redis::Barrier;
    ///
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// let mut  barrier = Barrier::new(1, "barrier_doc_test", client);
    /// let barrier_wait_result = barrier.wait();
    /// println!("{:?}", barrier_wait_result.is_leader());
    /// ```
    pub fn is_leader(&self) -> bool {
        self.0
    }
}

enum BarrierError {
    RedisError(redis::RedisError),
}

impl Barrier {
    pub fn new(num: usize, key: &str, client: redis::Client) -> Self {
        let mut conn = client.get_connection().unwrap();

        let uuid = redis::Script::new(UUID_SCRIPT)
            .arg(&key)
            .arg(&num)
            .invoke::<usize>(&mut conn)
            .expect("Failed to create barrier");

        Barrier {
            uuid: uuid,
            num,
            key: key.to_string(),
            _client: client,
            conn: Some(conn),
        }
    }

    /// Blocks the current thread until all threads have rendezvoused here.
    ///
    /// Barriers are re-usable after all threads have rendezvoused once, and can
    /// be used continuously.
    ///
    /// A single (arbitrary) thread will receive a [`BarrierWaitResult`] that
    /// returns `true` from [`BarrierWaitResult::is_leader()`] when returning
    /// from this function, and all other threads will receive a result that
    /// will return `false` from [`BarrierWaitResult::is_leader()`].
    ///
    /// The barrier needs to be mutable, because it guarantees that the barrier is only used once in thread.
    /// If you want to synchronize threads, you need to create a new barrier for each thread, so it has its own uuid.
    ///
    /// # Examples
    ///
    /// ```
    /// use dtypes::redis::Barrier;
    /// use std::thread;
    ///
    /// let n = 10;
    /// let mut handles = Vec::with_capacity(n);
    /// let client = redis::Client::open("redis://localhost:6379").unwrap();
    /// for _ in 0..n {
    ///     // The same messages will be printed together.
    ///     // You will NOT see any interleaving.
    ///     let mut barrier = Barrier::new(n, "barrier_doc_test2", client.clone());
    ///     handles.push(thread::spawn(move|| {
    ///         println!("before wait");
    ///         barrier.wait();
    ///         println!("after wait");
    ///     }));
    /// }
    /// // Wait for other threads to finish.
    /// for handle in handles {
    ///     handle.join().unwrap();
    /// }
    /// ```
    pub fn wait(&mut self) -> BarrierWaitResult {
        let mut conn = self.conn.take().unwrap();
        let timeout = 2;

        let mut status = RedisBarrierStatus::Waiting;
        while status == RedisBarrierStatus::Waiting {
            status = redis::Script::new(WAITING_SCRIPT)
                .arg(&self.key)
                .arg(self.uuid)
                .arg(self.num)
                .arg(timeout)
                .invoke::<u8>(&mut conn)
                .expect("Failed to wait for barrier")
                .into();
        }
        self.conn = Some(conn);

        if status == RedisBarrierStatus::Leader {
            BarrierWaitResult(true)
        } else {
            BarrierWaitResult(false)
        }
    }
}

impl Drop for Barrier {
    fn drop(&mut self) {
        let mut conn = self.conn.take().unwrap();
        redis::Script::new(RESET_SCRIPT)
            .arg(&self.key)
            .arg(self.uuid)
            .arg(self.num)
            .invoke::<()>(&mut conn)
            .expect("Failed to reset barrier");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::thread::sleep;

    #[test]
    fn test_barrier_leader() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        let mut barrier = Barrier::new(1, "barrier_test_leader", client);
        let barrier_wait_result = barrier.wait();
        assert!(barrier_wait_result.is_leader());
    }

    #[test]
    fn test_barrier_not_leader() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();

        let mut barrier = Barrier::new(2, "barrier_test_notleader", client.clone());

        let h1 = thread::spawn(move || {
            let mut barrier = Barrier::new(2, "barrier_test_notleader", client);
            let barrier_wait_result = barrier.wait();
            assert!(!barrier_wait_result.is_leader());
        });

        let h2 = thread::spawn(move || {
            sleep(std::time::Duration::from_millis(1000));
            let barrier_wait_result = barrier.wait();
            assert!(barrier_wait_result.is_leader());
        });

        h1.join().unwrap();
        h2.join().unwrap();
    }

    #[test]
    fn test_barrier_slow_check() {
        let n = 10;
        let mut handles = Vec::with_capacity(n);
        let client = redis::Client::open("redis://localhost:6379").unwrap();
        for _ in 0..n {
            // The same messages will be printed together.
            // You will NOT see any interleaving.
            let mut barrier = Barrier::new(n, "barrier_doc_test2", client.clone());
            handles.push(thread::spawn(move || barrier.wait().is_leader()));
        }
        // Wait for other threads to finish.
        assert_eq!(
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .map(|x| if x { 1 } else { 0 })
                .sum::<i32>(),
            1
        );
    }

    #[test]
    fn test_barrier_reuse() {
        let client = redis::Client::open("redis://localhost:6379").unwrap();

        let mut barrier = Barrier::new(1, "barrier_test_reuse", client.clone());
        barrier.wait();
        let mut barrier = Barrier::new(1, "barrier_test_reuse", client.clone());
        barrier.wait();
    }
}
