use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, RedisError};

pub type RedisPool = ConnectionManager;

/// Create a Redis connection pool
pub async fn create_redis_pool(redis_url: &str) -> Result<RedisPool, RedisError> {
    let client = Client::open(redis_url)?;
    ConnectionManager::new(client).await
}

/// Cache key constants for resolved members
pub mod cache_keys {
    use uuid::Uuid;

    /// Generate cache key for resolved members of a group
    pub fn resolved_members(group_id: Uuid) -> String {
        format!("group:resolved:{}", group_id)
    }

    /// Pattern to match all resolved members cache keys
    pub const RESOLVED_MEMBERS_PATTERN: &str = "group:resolved:*";

    /// Generate cache key for schedule result
    pub fn schedule_result(schedule_id: Uuid) -> String {
        format!("schedule:result:{}", schedule_id)
    }

    /// Pattern to match all schedule result cache keys
    pub const SCHEDULE_RESULT_PATTERN: &str = "schedule:result:*";
}

/// Cache TTL constants (in seconds)
pub mod cache_ttl {
    /// TTL for resolved members cache (5 minutes)
    pub const RESOLVED_MEMBERS: u64 = 300;

    /// TTL for schedule result cache (1 hour)
    pub const SCHEDULE_RESULT: u64 = 3600;
}

/// Invalidate a specific cache key
pub async fn invalidate_cache(redis_conn: &mut ConnectionManager, key: &str) {
    let _: Result<(), _> = redis_conn.del(key).await;
}

/// Invalidate multiple cache keys by pattern
pub async fn invalidate_cache_pattern(redis_conn: &mut ConnectionManager, pattern: &str) {
    let keys: Result<Vec<String>, _> = redis_conn.keys(pattern).await;
    if let Ok(keys) = keys {
        if !keys.is_empty() {
            let _: Result<(), _> = redis::cmd("DEL").arg(&keys).query_async(redis_conn).await;
        }
    }
}

/// Get a cached value
pub async fn get_cached<T: serde::de::DeserializeOwned>(
    redis_conn: &mut ConnectionManager,
    key: &str,
) -> Option<T> {
    let cached: Result<String, _> = redis_conn.get(key).await;
    if let Ok(cached_data) = cached {
        serde_json::from_str(&cached_data).ok()
    } else {
        None
    }
}

/// Set a cached value with TTL
pub async fn set_cached<T: serde::Serialize>(
    redis_conn: &mut ConnectionManager,
    key: &str,
    value: &T,
    ttl_seconds: u64,
) {
    if let Ok(json) = serde_json::to_string(value) {
        let _: Result<(), _> = redis_conn.set_ex(key, json, ttl_seconds).await;
    }
}
