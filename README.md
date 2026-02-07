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

## Prerequisites
- Rust 1.88 or higher
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

### 4. Import Sample Data (Optional)

```bash
# Import staff data
curl -X POST http://localhost:8080/api/v1/batch/staff \
  -H "Content-Type: application/json" \
  -d @data/staff.json

# Import groups
curl -X POST http://localhost:8080/api/v1/groups/groups \
  -H "Content-Type: application/json" \
  -d @data/groups.json
  
#import memberships
curl -X POST http://localhost:8080/api/v1/memberships/batch \
  -H "Content-Type: application/json" \
  -d @data/memberships.json
```

### 5. Access API Documentation

- **Data Service Swagger UI**: http://localhost:8080/swagger-ui
- **Scheduling Service Swagger UI**: http://localhost:8081/swagger-ui

## API Documentation

### Data Service Endpoints

#### Staff

- `POST /api/v1/staff` - Create a new staff member
- `GET /api/v1/staff` - List all staff (paginated)
- `GET /api/v1/staff/{id}` - Get staff by ID
- `PUT /api/v1/staff/{id}` - Update staff
- `DELETE /api/v1/staff/{id}` - Delete staff
- `POST /api/v1/staff/batch-import` - Batch import staff from JSON

#### Groups

- `POST /api/v1/groups` - Create a new group
- `GET /api/v1/groups` - List all groups (paginated)
- `GET /api/v1/groups/{id}` - Get group by ID
- `GET /api/v1/groups/{id}/resolved-members` - Get all active staff in group and descendants (**cached**)
- `PUT /api/v1/groups/{id}` - Update group
- `DELETE /api/v1/groups/{id}` - Delete group
- `POST /api/v1/groups/batch-import` - Batch import groups from JSON

#### Memberships

- `POST /api/v1/memberships` - Add staff to group
- `DELETE /api/v1/memberships/{id}` - Remove staff from group

### Scheduling Service Endpoints

- `POST /api/v1/schedules` - Submit a new schedule job (202 Accepted)
- `GET /api/v1/schedules/{schedule_id}/status` - Get job status
- `GET /api/v1/schedules/{schedule_id}` - Get completed schedule result (**cached**)

### Example: Generate a Schedule

```bash
# 1. Submit schedule job
curl -X POST http://localhost:8081/api/v1/schedules \
  -H "Content-Type: application/json" \
  -d '{
    "staff_group_id": "123e4567-e89b-12d3-a456-426614174000",
    "period_begin_date": "2024-01-15"
  }'
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
