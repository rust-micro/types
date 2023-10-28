/// The read lock script.
///
/// Checks if the writer list besides the key is empty or the lock is set.
/// If there are no writers, the uuid will be set as a reader and returns true.
/// Returns false otherwise.
///
/// The timeout will be used for the reader lock. You need to retry to get the lock again if you want to keep it.
/// But if a writer comes in scope, the reader lock will be dropped after the timeout and you have to wait.
///
/// Takes 3 arguments:
/// 1. The key to lock
/// 2. The uuid of the lock
/// 3. The timeout in seconds
pub const READER_LOCK: &str = r#"
if redis.call("exists", ARGV[1] .. ":lock") == 1 then
    return 0
end

local res = redis.call("scan", 0, "match", ARGV[1] .. ":writer_waiting_list:*")
if next(res[2]) == nil then
    redis.call("set", ARGV[1] .. ":reader_locks:" .. ARGV[2], 1, "ex", ARGV[3])
    return 1
end
return 0
"#;

/// The read lock drop script.
///
/// Removes the uuid from the reader list.
///
/// Takes 2 arguments:
/// 1. The key to lock
/// 2. The uuid of the lock
pub const READER_LOCK_DROP: &str = r#"
redis.call("del", ARGV[1] .. ":reader_locks:" .. ARGV[2])
return 1
"#;

/// The writer lock script.
///
/// Checks if the reader list besides the key is empty.
/// Also add the uuid to the writer waiting list.
/// If there are no readers, the uuid will be set as the lock and returns true.
/// Returns false otherwise.
///
/// The timeout will also be used for the waiting ticket, so if you wait too long, your intention will be dropped and reader can be acquired.
/// So be sure to request the lock again fast enough.
///
/// Takes 3 arguments:
/// 1. The key to lock
/// 2. The uuid of the lock
/// 3. The timeout in seconds for waiting
pub const WRITER_LOCK: &str = r#"
redis.call("setex", ARGV[1] .. ":writer_waiting_list:" .. ARGV[2], ARGV[3], 1)
if redis.call("exists", ARGV[1] .. ":lock") == 1 then
    return 0
end

return redis.call("set", ARGV[1] .. ":lock", ARGV[2], "nx", "ex", ARGV[3])
"#;

/// The writer lock drop script.
///
/// Removes the uuid from the writer list.
///
/// Takes 2 arguments:
/// 1. The key to lock
/// 2. The uuid of the lock
pub const WRITER_LOCK_DROP: &str = r#"
redis.call("del", ARGV[1] .. ":writer_waiting_list:" .. ARGV[2])
if redis.call("get", ARGV[1] .. ":lock") == ARGV[2] then
    redis.call("del", ARGV[1] .. ":lock")
end
return 1
"#;

/// The uuid script.
///
/// Increments the uuid counter and returns the new value.
///
/// Takes 1 argument:
/// 1. The key to lock
pub const UUID_SCRIPT: &str = r#"
return redis.call("INCR", ARGV[1] .. ":lock_counter")
"#;

/// The read script.
///
/// Reads the value from the key, only if the uuid is in reader list or if the lock is equal to uuid.
///
/// Takes 2 argument:
/// 1. The key to read
/// 2. The uuid of the lock
pub const LOAD_SCRIPT: &str = r#"
if redis.call("get", ARGV[1] .. ":lock") == ARGV[2] then
    return redis.call("get", ARGV[1])
end
if redis.call("exists", ARGV[1] .. ":reader_locks:" .. ARGV[2]) then
    return redis.call("get", ARGV[1])
end
"#;

/// The store script.
///
/// Stores the value to the key, only if the uuid is in lock.
///
/// Takes 3 arguments:
/// 1. The key to store
/// 2. The uuid of the lock
/// 3. The value to store
pub const STORE_SCRIPT: &str = r#"
if redis.call("get", ARGV[1] .. ":lock") == ARGV[2] then
    redis.call("set", ARGV[1], ARGV[3])
    return 1
end
return 0
"#;
