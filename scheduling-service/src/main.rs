mod api;
mod domain;
mod infrastructure;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use infrastructure::{config::Settings, database};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "scheduling_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Scheduling Service...");

    // Load configuration
    let settings = Settings::new()?;
    tracing::info!("Configuration loaded: {:?}", settings);

    // Initialize database pool
    let db_pool =
        database::create_pool(&settings.database.url, settings.database.max_connections).await?;
    tracing::info!("Database connection pool created");

    // Run migrations
    database::run_migrations(&db_pool).await?;
    tracing::info!("Database migrations completed");

    // Create router
    let app = api::create_router(db_pool);

    // Start server
    let listener = tokio::net::TcpListener::bind(settings.server_address()).await?;
    let addr = listener.local_addr()?;
    tracing::info!("Scheduling Service listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
