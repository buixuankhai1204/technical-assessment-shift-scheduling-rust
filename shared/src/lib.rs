pub mod error;
pub mod pagination;
pub mod types;

// Re-export commonly used items
pub use error::{DomainError, DomainResult};
pub use pagination::{PaginatedResponse, PaginationParams};
pub use types::{Identifiable, JobStatus, ShiftType, StaffStatus, Timestamped};
