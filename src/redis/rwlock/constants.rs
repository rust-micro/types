/// The read lock script.
///
/// Checks if the writer list besides the key is empty.
/// If it is, the uuid is added to the reader list and true is returned.
/// Returns false otherwise.
///
/// Takes 2 arguments:
/// 1. The key to lock
/// 2. The uuid of the lock
pub const READER_LOCK: &str = r#"
local writer_len = redis.call("LLEN", ARGV[1] .. ":writer")
if writer_len == 0 then
    redis.call("RPUSH", ARGV[1] .. ":reader", ARGV[2])
    return true
end
return false
"#;

/// The read lock drop script.
///
/// Removes the uuid from the reader list.
///
/// Takes 2 arguments:
/// 1. The key to lock
/// 2. The uuid of the lock
pub const READER_LOCK_DROP: &str = r#"
local reader_len = redis.call("LLEN", ARGV[1] .. ":reader")
if reader_len > 0 then
    redis.call("LREM", ARGV[1] .. ":reader", 1, ARGV[2])
    return true
end
return false
"#;

/// The writer lock script.
///
/// Checks if the reader and writer list besides the key are empty.
/// If they are, the uuid is added to the writer list and true is returned.
/// Returns false otherwise.
///
/// Takes 2 arguments:
/// 1. The key to lock
/// 2. The uuid of the lock
pub const WRITER_LOCK: &str = r#"
local reader_len = redis.call("LLEN", ARGV[1] .. ":reader")
local writer_len = redis.call("LLEN", ARGV[1] .. ":writer")
if reader_len == 0 and writer_len == 0 then
    redis.call("RPUSH", ARGV[1] .. ":writer", ARGV[2])
    return true
end
return false
"#;

/// The writer lock drop script.
///
/// Removes the uuid from the writer list.
///
/// Takes 2 arguments:
/// 1. The key to lock
/// 2. The uuid of the lock
pub const WRITER_LOCK_DROP: &str = r#"
local writer_len = redis.call("LLEN", ARGV[1] .. ":writer")
if writer_len > 0 then
    redis.call("LREM", ARGV[1] .. ":writer", 1, ARGV[2])
    return true
end
return false
"#;

/// The uuid script.
///
/// Increments the uuid counter and returns the new value.
///
/// Takes 1 argument:
/// 1. The key to lock
pub const UUID_SCRIPT: &str = r#"
redis.call("INCR", ARGV[1] .. ":uuid")
return redis.call("GET", ARGV[1] .. ":uuid")
"#;

/// The read script.
///
/// Reads the value from the key, only if the uuid is in reader list or if it is the single entry in the writer list.
///
/// Takes 2 argument:
/// 1. The key to read
/// 2. The uuid of the lock
pub const READ_SCRIPT: &str = r#"
local function contains(table, val)
    for i=1,#table do
        if table[i] == val then 
            return true
        end
    end
    return false
end

local readers = redis.call("LRANGE", ARGV[1] .. ":reader" , 0, -1)
local writers = redis.call("LRANGE", ARGV[1] .. ":writer" , 0, -1)

if contains(readers, ARGV[2]) or (#writers == 1 and writers[1] == ARGV[2]) then
    return redis.call("GET", ARGV[1])
end
"#;

/// The store script.
///
/// Stores the value to the key, only if the uuid is in writer list and the list is only one.
///
/// Takes 3 arguments:
/// 1. The key to store
/// 2. The uuid of the lock
/// 3. The value to store
pub const STORE_SCRIPT: &str = r#"
local writers = redis.call("LRANGE", ARGV[1] .. ":writer" , 0, -1)
if #writers == 1 and writers[1] == ARGV[2] then
    redis.call("SET", ARGV[1], ARGV[3])
    return true
end
return false
"#;
