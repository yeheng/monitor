use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder();
        
        cfg = cfg
            .set_default("database.host", "localhost")?
            .set_default("database.port", 5432)?
            .set_default("database.max_connections", 10)?
            .set_default("redis.max_connections", 10)?
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("auth.jwt_expiration", 86400)?;

        if let Ok(database_url) = env::var("DATABASE_URL") {
            cfg = cfg.set_override("database.url", database_url)?;
        } else {
            cfg = cfg
                .set_override("database.username", env::var("DATABASE_USERNAME").unwrap_or_else(|_| "monitor".to_string()))?
                .set_override("database.password", env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "password".to_string()))?
                .set_override("database.database", env::var("DATABASE_NAME").unwrap_or_else(|_| "monitor".to_string()))?;
        }

        cfg = cfg
            .set_override("redis.url", env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()))?
            .set_override("auth.jwt_secret", env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string()))?;

        if let Ok(port) = env::var("PORT") {
            cfg = cfg.set_override("server.port", port.parse::<u16>().unwrap_or(8080))?;
        }

        cfg.build()?.try_deserialize()
    }
}