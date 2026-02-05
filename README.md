# Shift Scheduling System

A comprehensive REST API-based shift scheduling system built with Rust, featuring asynchronous job processing, hierarchical group management, and Redis caching.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [API Documentation](#api-documentation)
- [Configuration](#configuration)
- [Testing](#testing)
- [Project Structure](#project-structure)

## Overview

This system consists of two microservices:

1. **Data Service** (`data-service`) - Manages staff and groups with CRUD operations and Redis caching
2. **Scheduling Service** (`scheduling-service`) - Generates shift schedules using a greedy algorithm with async job processing

The services communicate via HTTP and share common types through a `shared` crate.

## Architecture

### Clean Architecture

Both services follow Clean Architecture principles with three distinct layers:

- **API Layer** (`api/`) - HTTP handlers, routes, and API state
- **Domain Layer** (`domain/`) - Business logic, entities, and repository traits
- **Infrastructure Layer** (`infrastructure/`) - Repository implementations, database, and external clients

### Technology Stack

- **Language**: Rust 2021 Edition
- **Web Framework**: Axum
- **Async Runtime**: Tokio
- **Database**: PostgreSQL with sqlx
- **Caching**: Redis
- **API Documentation**: OpenAPI 3.0 with utoipa
- **Containerization**: Docker & Docker Compose

## Features

### Data Service

- **Staff Management**
  - CRUD operations for staff members
  - Staff status (ACTIVE/INACTIVE)
  - Pagination support
  - Batch import from JSON

- **Group Management**
  - Hierarchical group structure (parent-child relationships)
  - CRUD operations
  - Batch import from JSON

- **Group Membership**
  - Many-to-many relationship between staff and groups
  - Add/remove staff from groups

- **Hierarchical Group Resolution**
  - GET endpoint that resolves all descendants of a group
  - Returns all active staff members from group and its subgroups
  - Optimized with batch queries to avoid N+1 problem

- **Redis Caching**
  - All GET endpoints cached with 5-minute TTL
  - Automatic cache invalidation on mutations

### Scheduling Service

- **Async Schedule Generation**
  - Submit schedule jobs via REST API
  - Background processing with Tokio channels
  - 28-day scheduling period starting from Monday

- **Scheduling Algorithm**
  - Greedy algorithm for shift assignments
  - Supports MORNING, EVENING, and DAY_OFF shifts
  - Configurable scheduling rules:
    - Min/max days off per week (default: 1-2)
    - No morning shift after evening shift
    - Max daily shift difference (default: 1)

- **Job Status Tracking**
  - PENDING → PROCESSING → COMPLETED/FAILED
  - Error message capture on failure
  - Timestamp tracking (created, updated, completed)

- **Schedule Result Retrieval**
  - GET endpoint for completed schedules
  - Returns all shift assignments for the 28-day period

## Prerequisites

- Rust 1.75 or higher
- Docker and Docker Compose
- PostgreSQL 15 (via Docker)
- Redis 7 (via Docker)

## Getting Started

### 1. Clone the Repository

```bash
git clone <repository-url>
cd technical-assessment-shift-scheduling-rust
```

### 2. Start Services with Docker Compose

```bash
docker-compose up -d
```

This will start:
- PostgreSQL on port 5432 and 5433
- Redis on port 6379
- Data Service on port 8080
- Scheduling Service on port 8081

### 3. Run Database Migrations

Migrations run automatically when services start, but you can run them manually:

```bash
# Data Service migrations
cd data-service
sqlx migrate run

# Scheduling Service migrations
cd ../scheduling-service
sqlx migrate run
```

### 4. Import Sample Data (Optional)

```bash
# Import staff data
curl -X POST http://localhost:8080/api/v1/staff/batch-import \
  -H "Content-Type: application/json" \
  -d @data/staff.json

# Import groups
curl -X POST http://localhost:8080/api/v1/groups/batch-import \
  -H "Content-Type: application/json" \
  -d @data/groups.json
```

### 5. Access API Documentation

- **Data Service Swagger UI**: http://localhost:8080/swagger-ui
- **Scheduling Service Swagger UI**: http://localhost:8081/swagger-ui

## API Documentation

### Data Service Endpoints

#### Staff

- `POST /api/v1/staff` - Create a new staff member
- `GET /api/v1/staff` - List all staff (paginated, cached)
- `GET /api/v1/staff/{id}` - Get staff by ID (cached)
- `PUT /api/v1/staff/{id}` - Update staff
- `DELETE /api/v1/staff/{id}` - Delete staff
- `POST /api/v1/staff/batch-import` - Batch import staff from JSON

#### Groups

- `POST /api/v1/groups` - Create a new group
- `GET /api/v1/groups` - List all groups (paginated, cached)
- `GET /api/v1/groups/{id}` - Get group by ID (cached)
- `GET /api/v1/groups/{id}/members` - Get all active staff in group and descendants (cached)
- `PUT /api/v1/groups/{id}` - Update group
- `DELETE /api/v1/groups/{id}` - Delete group
- `POST /api/v1/groups/batch-import` - Batch import groups from JSON

#### Memberships

- `POST /api/v1/memberships` - Add staff to group
- `GET /api/v1/memberships/group/{group_id}` - Get memberships by group (cached)
- `GET /api/v1/memberships/staff/{staff_id}` - Get memberships by staff (cached)
- `DELETE /api/v1/memberships/{id}` - Remove staff from group

### Scheduling Service Endpoints

- `POST /api/v1/schedules` - Submit a new schedule job (202 Accepted)
- `GET /api/v1/schedules/{schedule_id}/status` - Get job status
- `GET /api/v1/schedules/{schedule_id}` - Get completed schedule result

### Example: Generate a Schedule

```bash
# 1. Submit schedule job
curl -X POST http://localhost:8081/api/v1/schedules \
  -H "Content-Type: application/json" \
  -d '{
    "staff_group_id": "123e4567-e89b-12d3-a456-426614174000",
    "period_begin_date": "2024-01-15"
  }'

# Response:
# {
#   "schedule_id": "987f6543-e21a-12d3-a456-426614174000",
#   "status": "PENDING"
# }

# 2. Check status
curl http://localhost:8081/api/v1/schedules/987f6543-e21a-12d3-a456-426614174000/status

# 3. Get completed schedule
curl http://localhost:8081/api/v1/schedules/987f6543-e21a-12d3-a456-426614174000
```

## Configuration

### Data Service Configuration

File: `data-service/config/default.toml`

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgresql://postgres:postgres@localhost:5432/data_service_db"
max_connections = 10

[redis]
url = "redis://localhost:6379"
```

### Scheduling Service Configuration

File: `scheduling-service/config/default.toml`

```toml
[server]
host = "0.0.0.0"
port = 8081

[database]
url = "postgresql://postgres:postgres@localhost:5433/scheduling_service_db"
max_connections = 10

[data_service]
host = "data-service"
port = 8080

[scheduling]
min_days_off_per_week = 1
max_days_off_per_week = 2
max_daily_shift_difference = 1
```

### Environment Variables

Override config with environment variables using `APP__` prefix:

```bash
export APP__SERVER__PORT=9090
export APP__DATABASE__URL="postgresql://..."
```

## Testing

### Run Tests

```bash
# Test all workspaces
cargo test

# Test specific service
cd data-service && cargo test
cd scheduling-service && cargo test

# Test with output
cargo test -- --nocapture
```

### Run Clippy

```bash
cargo clippy --all-targets --all-features
```

### Format Code

```bash
cargo fmt --all
```

## Project Structure

```
.
├── data-service/           # Data management service
│   ├── src/
│   │   ├── api/           # HTTP handlers and routes
│   │   ├── domain/        # Business logic and entities
│   │   └── infrastructure/# Repository implementations
│   ├── migrations/        # SQL migrations
│   └── config/           # Configuration files
│
├── scheduling-service/    # Scheduling service
│   ├── src/
│   │   ├── api/          # HTTP handlers and routes
│   │   ├── domain/       # Scheduling logic
│   │   └── infrastructure/# Database and HTTP client
│   ├── migrations/       # SQL migrations
│   └── config/          # Configuration files
│
├── shared/               # Shared types and utilities
│   └── src/
│       ├── error.rs     # Domain errors
│       ├── types.rs     # Common enums
│       └── pagination.rs# Pagination support
│
├── data/                # Sample data files
│   ├── staff.json
│   └── groups.json
│
├── docker-compose.yml   # Docker orchestration
└── Cargo.toml          # Workspace definition
```

## Key Implementation Details

### N+1 Query Prevention

The Data Service implements batch queries and concurrent operations using `futures::try_join_all` to avoid N+1 query problems:

```rust
// Example from GroupService::get_resolved_members
let membership_futures = group_ids
    .iter()
    .map(|gid| self.membership_repo.find_by_group_id(*gid));
let membership_results = try_join_all(membership_futures).await?;

// Single batch query for all staff
let all_staff = self.staff_repo.find_by_ids(staff_ids).await?;
```

### Dependency Inversion

Repository traits enable dependency inversion and testability:

```rust
#[async_trait]
pub trait StaffRepository: Send + Sync {
    async fn create(&self, staff: Staff) -> DomainResult<Staff>;
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Staff>>;
    // ... more methods
}
```

### Async Job Processing

The Scheduling Service uses Tokio channels for background job processing:

```rust
// In main.rs
let processor = Arc::new(ScheduleProcessor::new(...));
let (schedule_sender, processor_handle) = processor.start();

// Handlers submit jobs via channel
schedule_sender.send(request).await?;
```

## Troubleshooting

### Database Connection Issues

```bash
# Check PostgreSQL is running
docker ps | grep postgres

# Test connection
psql postgresql://postgres:postgres@localhost:5432/data_service_db
```

### Redis Connection Issues

```bash
# Check Redis is running
docker ps | grep redis

# Test connection
redis-cli -h localhost ping
```

### Service Logs

```bash
# View service logs
docker-compose logs -f data-service
docker-compose logs -f scheduling-service
```

## Performance Considerations

- **Redis Caching**: 5-minute TTL on all GET endpoints reduces database load
- **Batch Queries**: `find_by_ids` methods reduce round trips
- **Concurrent Operations**: `try_join_all` parallelizes independent queries
- **Connection Pooling**: sqlx connection pools optimize database connections
- **Async Processing**: Scheduling jobs don't block API requests

## License

MIT License

## Authors

Technical Assessment Implementation - 2024