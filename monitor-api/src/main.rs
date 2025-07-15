use monitor_core::{
    Result,
    auth::AuthService,
    cache::create_redis_pool,
    config::Config,
    db::{create_pool, run_migrations},
    logging,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

mod server;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init_logging();

    let config = Config::from_env()?;
    info!("Starting Monitor API server with config: {:?}", config);

    let db_pool = create_pool(&config.database).await?;
    info!("Database connection established");

    run_migrations(&db_pool).await?;
    info!("Database migrations completed");

    let redis_pool = create_redis_pool(&config.redis).await?;
    info!("Redis connection established");

    let auth_service = AuthService::new(config.auth.jwt_secret.clone(), config.auth.jwt_expiration);

    let state = Arc::new(server::AppState {
        db: db_pool,
        redis: redis_pool,
        auth: auth_service,
        config: config.clone(),
    });

    let app = server::create_app(state).await;

    let listener = TcpListener::bind(&format!("{}:{}", config.server.host, config.server.port))
        .await
        .expect("init tcp listener failed");

    info!(
        "Server listening on {}:{}",
        config.server.host, config.server.port
    );

    axum::serve(listener, app).await?;

    Ok(())
}
