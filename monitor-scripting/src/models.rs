use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ScriptResult {
    pub success: bool,
    pub result: Option<Value>,
    pub error: Option<Value>,
    pub execution_time_ms: u64,
    pub memory_usage: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationContext {
    pub status_code: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
    pub response_time: u64,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub passed: bool,
    pub message: String,
    pub details: Option<Value>,
    pub error_details: Option<Value>,
    pub execution_time_ms: u64,
}
