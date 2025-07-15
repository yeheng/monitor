use sqlx::{PgPool, Pool, Postgres};
use crate::{config::DatabaseConfig, error::Result};

pub type DatabasePool = Pool<Postgres>;

pub async fn create_pool(config: &DatabaseConfig) -> Result<DatabasePool> {
    let connection_string = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.username,
        config.password,
        config.host,
        config.port,
        config.database
    );

    let pool = PgPool::connect(&connection_string).await?;
    
    Ok(pool)
}

pub async fn run_migrations(pool: &DatabasePool) -> Result<()> {
    sqlx::migrate!("../monitor-core/migrations").run(pool).await?;
    Ok(())
}