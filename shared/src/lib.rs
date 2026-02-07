pub mod cache;
pub mod error;
pub mod pagination;
pub mod response;
pub mod types;

// Re-export commonly used items
pub use cache::{
    cache_keys, cache_ttl, create_redis_pool, get_cached, invalidate_cache,
    invalidate_cache_pattern, set_cached, RedisPool,
};
pub use error::{DomainError, DomainResult};
pub use pagination::{PaginatedResponse, PaginationParams};
pub use response::ApiResponse;
pub use types::{Identifiable, JobStatus, ShiftType, StaffStatus, Timestamped};
