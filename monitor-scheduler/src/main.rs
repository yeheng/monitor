use monitor_core::{
    config::Config,
    db::{create_pool, run_migrations},
    logging,
    Result,
};
use tracing::info;

mod scheduler;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init_logging();
    
    let config = Config::from_env()?;
    info!("Starting Monitor Scheduler with config: {:?}", config);

    let db_pool = create_pool(&config.database).await?;
    info!("Database connection established");

    run_migrations(&db_pool).await?;
    info!("Database migrations completed");

    let mut scheduler = scheduler::MonitorScheduler::new(db_pool).await?;
    
    scheduler.start().await?;
    scheduler.load_and_schedule_monitors().await?;
    
    info!("Monitor scheduler is running. Press Ctrl+C to stop.");
    
    tokio::signal::ctrl_c().await?;
    
    info!("Shutdown signal received");
    scheduler.stop().await?;
    
    Ok(())
}
