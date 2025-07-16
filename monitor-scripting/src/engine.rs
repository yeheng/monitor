use monitor_core::{Error, Result};
use rquickjs::{Context, Runtime, Value as JsValue};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

pub struct ScriptEngine {
    runtime: Runtime,
    timeout: Duration,
}

impl ScriptEngine {
    pub fn new() -> Result<Self> {
        Self::with_timeout(Duration::from_secs(30))
    }

    pub fn with_timeout(timeout: Duration) -> Result<Self> {
        let runtime = Runtime::new()
            .map_err(|e| Error::script_execution(format!("Failed to create runtime: {}", e)))?;

        Ok(Self { runtime, timeout })
    }

    pub async fn execute_script(&self, script: &str, context_data: &Value) -> Result<ScriptResult> {
        let start_time = Instant::now();
        let script_with_metadata = self.wrap_script_with_metadata(script);

        let ctx = Context::full(&self.runtime)
            .map_err(|e| Error::script_execution(format!("Failed to create context: {}", e)))?;

        let result: Result<ScriptResult> = ctx.with(|ctx| {
            // Set up the context with monitor data
            let global = ctx.globals();

            // Add context data
            if let Ok(context_str) = serde_json::to_string(context_data) {
                let _ = ctx.eval::<(), _>(format!("const context = {}", context_str));
            }

            // Add enhanced utility functions
            let utility_script = self.get_utility_functions();
            if let Err(e) = ctx.eval::<(), _>(utility_script.as_str()) {
                return Err(Error::script_execution(format!(
                    "Failed to load utilities: {}",
                    e
                )));
            }

            // Set up timeout checking
            let _ = global.set("__start_time", start_time.elapsed().as_millis() as f64);
            let timeout_ms = self.timeout.as_millis() as f64;
            let _ = global.set("__timeout_ms", timeout_ms);

            // Execute the user script with timeout checking
            match ctx.eval::<JsValue, _>(script_with_metadata.as_str()) {
                Ok(result) => {
                    let execution_time = start_time.elapsed();
                    let result_value = js_value_to_serde_value(&result)?;
                    Ok(ScriptResult {
                        success: true,
                        result: Some(result_value),
                        error: None,
                        execution_time_ms: execution_time.as_millis() as u64,
                        memory_usage: None, // Could be enhanced with memory tracking
                    })
                }
                Err(e) => {
                    let execution_time = start_time.elapsed();
                    let error_details = self.extract_detailed_error(&e, script);
                    Ok(ScriptResult {
                        success: false,
                        result: None,
                        error: Some(error_details),
                        execution_time_ms: execution_time.as_millis() as u64,
                        memory_usage: None,
                    })
                }
            }
        });

        result.map_err(|e| Error::script_execution(format!("Script execution failed: {}", e)))
    }

    fn wrap_script_with_metadata(&self, script: &str) -> String {
        // For simple expressions and single statements, don't wrap them
        let trimmed = script.trim();
        if trimmed.lines().count() <= 2
            && !trimmed.contains("function")
            && !trimmed.contains("var ")
            && !trimmed.contains("let ")
            && !trimmed.contains("const ")
        {
            return script.to_string();
        }

        format!(
            r#"
(function() {{
    // Timeout check wrapper
    function checkTimeout() {{
        const now = performance && performance.now ? performance.now() : Date.now();
        if (typeof __start_time !== 'undefined' && typeof __timeout_ms !== 'undefined') {{
            if ((now - __start_time) > __timeout_ms) {{
                throw new Error('Script execution timeout after ' + __timeout_ms + 'ms');
            }}
        }}
    }}
    
    // Add line tracking for better error reporting
    try {{
        checkTimeout();
        return (function() {{
            {script}
        }})();
    }} catch (error) {{
        // Re-throw with enhanced error information
        if (error.name === 'Error' && !error.line) {{
            error.line = 'unknown';
            error.column = 'unknown';
        }}
        throw error;
    }}
}})();
"#,
            script = script
        )
    }

    fn get_utility_functions(&self) -> String {
        // Load utility functions from an external file
        let utility_script = include_str!("utility_functions.js");
        utility_script.to_string()
    }

    fn extract_detailed_error(&self, error: &rquickjs::Error, original_script: &str) -> Value {
        match error {
            rquickjs::Error::Exception => {
                // Try to extract exception details if available
                json!({
                    "type": "exception",
                    "message": "JavaScript exception occurred",
                    "details": "Exception details not available in this context"
                })
            }
            _ => {
                if let Some(exception_info) =
                    self.parse_error_message(&error.to_string(), original_script)
                {
                    exception_info
                } else {
                    json!({
                        "type": "runtime_error",
                        "message": error.to_string(),
                        "script_preview": self.get_script_preview(original_script, None)
                    })
                }
            }
        }
    }

    fn parse_error_message(&self, error_msg: &str, script: &str) -> Option<Value> {
        // Try to extract line/column information from error message
        let _lines: Vec<&str> = script.lines().collect();

        // Look for common error patterns
        if error_msg.contains("SyntaxError") {
            return Some(json!({
                "type": "syntax_error",
                "message": error_msg,
                "script_preview": self.get_script_preview(script, None),
                "suggestion": "Check for missing semicolons, brackets, or invalid syntax"
            }));
        }

        if error_msg.contains("ReferenceError") {
            return Some(json!({
                "type": "reference_error",
                "message": error_msg,
                "script_preview": self.get_script_preview(script, None),
                "suggestion": "Check for undefined variables or functions"
            }));
        }

        if error_msg.contains("TypeError") {
            return Some(json!({
                "type": "type_error",
                "message": error_msg,
                "script_preview": self.get_script_preview(script, None),
                "suggestion": "Check for incorrect data types or null/undefined values"
            }));
        }

        None
    }

    fn get_script_preview(&self, script: &str, error_line: Option<usize>) -> Value {
        let lines: Vec<&str> = script.lines().collect();
        let total_lines = lines.len();

        let (start, end, highlight) = if let Some(err_line) = error_line {
            let start = err_line.saturating_sub(2);
            let end = std::cmp::min(err_line + 3, total_lines);
            (start, end, Some(err_line))
        } else {
            let preview_size = 10;
            let end = std::cmp::min(preview_size, total_lines);
            (0, end, None)
        };

        let preview_lines: Vec<Value> = lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let line_num = start + i + 1;
                json!({
                    "line": line_num,
                    "content": line,
                    "is_error": highlight.map_or(false, |h| h == line_num - 1)
                })
            })
            .collect();

        json!({
            "lines": preview_lines,
            "total_lines": total_lines,
            "showing_range": format!("{}-{}", start + 1, end)
        })
    }

    pub async fn execute_validation_script(
        &self,
        script: &str,
        response_data: &ValidationContext,
    ) -> Result<ValidationResult> {
        let context_json = serde_json::to_value(response_data)
            .map_err(|e| Error::script_execution(format!("Failed to serialize context: {}", e)))?;

        let script_result = self.execute_script(script, &context_json).await?;

        let (passed, message) = if script_result.success {
            // For validation scripts, we consider it passed if:
            // 1. No exception was thrown
            // 2. The result is truthy (if it's a boolean/value)
            let result_is_truthy = script_result
                .result
                .as_ref()
                .map(|v| match v {
                    Value::Bool(b) => *b,
                    Value::Null => false,
                    Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                    Value::String(s) => !s.is_empty(),
                    Value::Array(a) => !a.is_empty(),
                    Value::Object(_) => true,
                })
                .unwrap_or(true);

            (result_is_truthy, "Validation passed".to_string())
        } else {
            let error_message = script_result
                .error
                .as_ref()
                .and_then(|e| e.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("Script execution failed")
                .to_string();
            (false, error_message)
        };

        Ok(ValidationResult {
            passed,
            message,
            details: script_result.result,
            error_details: script_result.error,
            execution_time_ms: script_result.execution_time_ms,
        })
    }
}

fn js_value_to_serde_value(value: &JsValue) -> Result<Value> {
    if value.is_undefined() {
        return Ok(json!({"__type": "undefined"}));
    }
    if value.is_null() {
        return Ok(Value::Null);
    }
    if value.is_bool() {
        return Ok(Value::Bool(value.as_bool().unwrap()));
    }
    if value.is_number() {
        let num = value.as_number().unwrap();
        // Handle special numeric values
        if num.is_nan() {
            return Ok(json!({"__type": "NaN"}));
        }
        if num.is_infinite() {
            return Ok(json!({"__type": "Infinity", "positive": num.is_sign_positive()}));
        }
        return Ok(json!(num));
    }
    if value.is_string() {
        let s = value
            .as_string()
            .unwrap()
            .to_string()
            .map_err(|e| Error::script_execution(format!("Failed to convert string: {}", e)))?;
        return Ok(Value::String(s));
    }
    if value.is_array() {
        let array = value.as_array().unwrap();
        let mut vec = Vec::new();
        for item in array.iter() {
            let item = item.unwrap();
            vec.push(js_value_to_serde_value(&item)?);
        }
        return Ok(Value::Array(vec));
    }
    if value.is_function() {
        return Ok(json!({
            "__type": "function",
            "name": "function"
        }));
    }
    if value.is_object() {
        let obj = value.as_object().unwrap();
        let mut map = serde_json::Map::new();

        // Check for special object types
        if let Ok(constructor) = obj.get::<_, JsValue>("constructor") {
            if let Some(name) = constructor
                .as_object()
                .and_then(|c| c.get::<_, String>("name").ok())
            {
                match name.as_str() {
                    "Date" => {
                        return Ok(json!({
                            "__type": "Date",
                            "timestamp": "date_object"
                        }));
                    }
                    "RegExp" => {
                        return Ok(json!({
                            "__type": "RegExp",
                            "source": "regex_pattern"
                        }));
                    }
                    "Error" => {
                        let message = obj.get::<_, String>("message").unwrap_or_default();
                        let name = obj
                            .get::<_, String>("name")
                            .unwrap_or_else(|_| "Error".to_string());
                        return Ok(json!({
                            "__type": "Error",
                            "name": name,
                            "message": message
                        }));
                    }
                    _ => {}
                }
            }
        }

        // Handle regular objects
        for result in obj.props::<String, JsValue>() {
            let (key, val) = result.unwrap();
            map.insert(key, js_value_to_serde_value(&val)?);
        }
        return Ok(Value::Object(map));
    }
    if value.is_symbol() {
        return Ok(json!({
            "__type": "symbol",
            "description": "symbol"
        }));
    }

    // Fallback for unknown types
    Ok(json!({
        "__type": "unknown",
        "string_representation": format!("{:?}", value)
    }))
}

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

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default ScriptEngine")
    }
}
