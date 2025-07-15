use monitor_core::{logging, Result};
use tracing::info;

pub mod engine;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init_logging();
    
    info!("Monitor Scripting Engine Demo");
    
    let script_engine = engine::ScriptEngine::new()?;
    
    let context = serde_json::json!({
        "test": "Hello from scripting engine!"
    });
    
    let simple_script = r#"
        log("Script is running!");
        log("Context test value: " + context.test);
        "Script executed successfully"
    "#;
    
    match script_engine.execute_script(simple_script, &context).await {
        Ok(result) => {
            info!("Script execution result: {:?}", result);
        }
        Err(e) => {
            info!("Script execution failed: {}", e);
        }
    }
    
    Ok(())
}
