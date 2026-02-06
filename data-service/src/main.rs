mod api;
mod domain;
mod infrastructure;
mod presentation;

use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::AppState;
use domain::repositories::{GroupRepository, MembershipRepository, StaffRepository};
use infrastructure::{
    config::Settings,
    database, redis,
    repositories::{
        PostgresGroupRepository, PostgresMembershipRepository, PostgresStaffRepository,
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "data_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Data Service...");

    let settings = Settings::new()?;
    tracing::info!("Configuration loaded: {:?}", settings);

    let db_pool =
        database::create_pool(&settings.database.url, settings.database.max_connections).await?;
    tracing::info!("Database connection pool created");

    database::run_migrations(&db_pool).await?;
    tracing::info!("Database migrations completed");
    let redis_pool = redis::create_redis_pool(&settings.redis.url).await?;
    tracing::info!("Redis connection established");
    let staff_repo: Arc<dyn StaffRepository> =
        Arc::new(PostgresStaffRepository::new(db_pool.clone()));
    let group_repo: Arc<dyn GroupRepository> =
        Arc::new(PostgresGroupRepository::new(db_pool.clone()));
    let membership_repo: Arc<dyn MembershipRepository> =
        Arc::new(PostgresMembershipRepository::new(db_pool.clone()));

    tracing::info!("Repositories initialized");

    let app_state = AppState::new(
        staff_repo,
        group_repo,
        membership_repo,
        redis_pool,
    );

    let app = api::create_router(app_state);
    let listener = tokio::net::TcpListener::bind(settings.server_address()).await?;
    let addr = listener.local_addr()?;
    tracing::info!("Data Service listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
