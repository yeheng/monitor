use monitor_core::{logging, Config, Result};

#[tokio::main]
async fn main() -> Result<()> {
    logging::init_logging();
    
    let config = Config::from_env()?;
    tracing::info!("Monitor Core started with config: {:?}", config);
    
    Ok(())
}
