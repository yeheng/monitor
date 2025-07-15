use redis::Client;
use crate::{config::RedisConfig, error::Result};

pub type RedisPool = Client;

pub async fn create_redis_pool(config: &RedisConfig) -> Result<RedisPool> {
    let client = Client::open(config.url.as_str())?;
    Ok(client)
}