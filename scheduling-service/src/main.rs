mod api;
mod domain;
mod infrastructure;
mod presentation;

use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::AppState;
use domain::rules::{MaxDaysOffRule, MinDaysOffRule, NoMorningAfterEveningRule, Rule, ShiftBalanceRule};
use infrastructure::{
    config::Settings,
    database,
    http_client::DataServiceClient,
    repositories::{PostgresScheduleJobRepository, PostgresShiftAssignmentRepository},
    JobProcessor, ScheduleGenerator,
};

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

    // Initialize repositories
    let job_repo = Arc::new(PostgresScheduleJobRepository::new(db_pool.clone()));
    let assignment_repo = Arc::new(PostgresShiftAssignmentRepository::new(db_pool.clone()));
    tracing::info!("Repositories initialized");

    // Initialize data service client
    let data_service_url = format!(
        "http://{}:{}",
        settings.data_service.host, settings.data_service.port
    );
    let data_service_client = Arc::new(DataServiceClient::new(data_service_url));
    tracing::info!("Data service client initialized");

    // Create scheduling rules from config
    let rules: Vec<Arc<dyn Rule>> = vec![
        Arc::new(NoMorningAfterEveningRule::new()),
        Arc::new(MinDaysOffRule::new(settings.scheduling.min_days_off_per_week)),
        Arc::new(MaxDaysOffRule::new(settings.scheduling.max_days_off_per_week)),
        Arc::new(ShiftBalanceRule::new(settings.scheduling.max_daily_shift_difference)),
    ];
    tracing::info!("Scheduling rules configured");

    // Create schedule generator
    let scheduler = Arc::new(ScheduleGenerator::new(rules));

    // Create job processor
    let processor = Arc::new(JobProcessor::new(
        job_repo.clone(),
        assignment_repo.clone(),
        data_service_client,
        scheduler,
    ));

    // Start background processor
    let (schedule_sender, processor_handle) = processor.start();
    tracing::info!("Background schedule processor started");

    // Create application state
    let app_state = AppState::new(job_repo, assignment_repo, schedule_sender);

    // Create router
    let app = api::create_router(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind(settings.server_address()).await?;
    let addr = listener.local_addr()?;
    tracing::info!("Scheduling Service listening on {}", addr);

    // Serve with graceful shutdown
    let server = axum::serve(listener, app);

    tokio::select! {
        result = server => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received shutdown signal");
        }
    }

    // Wait for background processor to finish
    processor_handle.abort();
    tracing::info!("Scheduling Service shutdown complete");

    Ok(())
}
