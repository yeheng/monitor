use monitor_core::{Error, Result};
use rquickjs::{Context, Runtime, Value as JsValue, Exception};
use serde_json::{json, Value};
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
            if let Err(e) = ctx.eval::<(), _>(&utility_script) {
                return Err(Error::script_execution(format!("Failed to load utilities: {}", e)));
            }

            // Set up timeout checking
            let _ = global.set("__start_time", start_time.elapsed().as_millis() as f64);
            let timeout_ms = self.timeout.as_millis() as f64;
            let _ = global.set("__timeout_ms", timeout_ms);

            // Execute the user script with timeout checking
            match ctx.eval::<JsValue, _>(&script_with_metadata) {
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
        format!(
            r#"
(function() {{
    // Timeout check wrapper
    function checkTimeout() {{
        const now = performance.now ? performance.now() : Date.now();
        if ((now - __start_time) > __timeout_ms) {{
            throw new Error('Script execution timeout after ' + __timeout_ms + 'ms');
        }}
    }}
    
    // Override some globals to add timeout checking
    const originalSetTimeout = globalThis.setTimeout || function() {{}};
    globalThis.setTimeout = function(fn, delay) {{
        checkTimeout();
        return originalSetTimeout.call(this, function() {{
            checkTimeout();
            return fn.apply(this, arguments);
        }}, delay);
    }};
    
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
        r#"
// Enhanced logging with levels
function log(message, level = 'INFO') {
    const timestamp = new Date().toISOString();
    console.log(`[${timestamp}] [${level}] [Script] ${message}`);
}

function debug(message) { log(message, 'DEBUG'); }
function info(message) { log(message, 'INFO'); }
function warn(message) { log(message, 'WARN'); }
function error(message) { log(message, 'ERROR'); }

// Enhanced assertion functions
function assert(condition, message) {
    if (!condition) {
        const error = new Error(message || 'Assertion failed');
        error.name = 'AssertionError';
        throw error;
    }
    return true;
}

function expect(actual, expected, message) {
    if (actual !== expected) {
        const error = new Error(message || `Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
        error.name = 'ExpectationError';
        error.actual = actual;
        error.expected = expected;
        throw error;
    }
    return true;
}

// Type checking utilities
function assertType(value, expectedType, message) {
    const actualType = typeof value;
    if (actualType !== expectedType) {
        throw new Error(message || `Expected type ${expectedType}, got ${actualType}`);
    }
    return true;
}

function assertInstanceOf(value, constructor, message) {
    if (!(value instanceof constructor)) {
        throw new Error(message || `Expected instance of ${constructor.name}, got ${typeof value}`);
    }
    return true;
}

// HTTP response validation utilities
function assertStatus(statusCode, expected, message) {
    return expect(statusCode, expected, message || `Expected status ${expected}, got ${statusCode}`);
}

function assertStatusRange(statusCode, min, max, message) {
    if (statusCode < min || statusCode > max) {
        throw new Error(message || `Expected status between ${min}-${max}, got ${statusCode}`);
    }
    return true;
}

function assertContains(text, substring, message) {
    if (typeof text !== 'string' || !text.includes(substring)) {
        throw new Error(message || `Expected text to contain "${substring}"`);
    }
    return true;
}

function assertMatches(text, pattern, message) {
    const regex = pattern instanceof RegExp ? pattern : new RegExp(pattern);
    if (!regex.test(text)) {
        throw new Error(message || `Expected text to match pattern ${regex}`);
    }
    return true;
}

// JSON utilities with error handling
function parseJSON(text, defaultValue = null) {
    try {
        return JSON.parse(text);
    } catch (e) {
        if (defaultValue !== null) {
            return defaultValue;
        }
        throw new Error(`Invalid JSON: ${e.message}`);
    }
}

function assertValidJSON(text, message) {
    try {
        JSON.parse(text);
        return true;
    } catch (e) {
        throw new Error(message || `Invalid JSON: ${e.message}`);
    }
}

// Performance timing
const performance = globalThis.performance || {
    now: function() { return Date.now(); }
};

function time(label) {
    const start = performance.now();
    return {
        end: function() {
            const duration = performance.now() - start;
            log(`${label}: ${duration.toFixed(2)}ms`, 'TIMER');
            return duration;
        }
    };
}
"#.to_string()
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
                if let Some(exception_info) = self.parse_error_message(&error.to_string(), original_script) {
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
        let lines: Vec<&str> = script.lines().collect();
        
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
            let result_is_truthy = script_result.result
                .as_ref()
                .map(|v| {
                    match v {
                        Value::Bool(b) => *b,
                        Value::Null => false,
                        Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                        Value::String(s) => !s.is_empty(),
                        Value::Array(a) => !a.is_empty(),
                        Value::Object(_) => true,
                    }
                })
                .unwrap_or(true);
            
            (result_is_truthy, "Validation passed".to_string())
        } else {
            let error_message = script_result.error
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
            "name": value.as_function()
                .and_then(|f| f.name())
                .unwrap_or_else(|| "anonymous".to_string())
        }));
    }
    if value.is_object() {
        let obj = value.as_object().unwrap();
        let mut map = serde_json::Map::new();
        
        // Check for special object types
        if let Ok(constructor) = obj.get::<_, JsValue>("constructor") {
            if let Some(name) = constructor.as_object()
                .and_then(|c| c.get::<_, String>("name").ok()) {
                match name.as_str() {
                    "Date" => {
                        if let Ok(timestamp) = obj.call_method::<_, f64>("getTime", ()) {
                            return Ok(json!({
                                "__type": "Date",
                                "timestamp": timestamp,
                                "iso_string": obj.call_method::<_, String>("toISOString", ()).ok()
                            }));
                        }
                    }
                    "RegExp" => {
                        if let Ok(source) = obj.call_method::<_, String>("toString", ()) {
                            return Ok(json!({
                                "__type": "RegExp",
                                "source": source
                            }));
                        }
                    }
                    "Error" => {
                        let message = obj.get::<_, String>("message").unwrap_or_default();
                        let name = obj.get::<_, String>("name").unwrap_or_else(|_| "Error".to_string());
                        let stack = obj.get::<_, String>("stack").ok();
                        return Ok(json!({
                            "__type": "Error",
                            "name": name,
                            "message": message,
                            "stack": stack
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
            "description": value.as_symbol()
                .and_then(|s| s.description())
                .unwrap_or_else(|| "".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_simple_script_execution() {
        let engine = ScriptEngine::new().unwrap();
        let context = serde_json::json!({
            "test": "value"
        });

        let result = engine
            .execute_script("1 + 1", &context)
            .await
            .unwrap();
        assert!(result.success);
        assert_eq!(result.result, Some(serde_json::json!(2.0)));
    }
    
        #[tokio::test]
        async fn test_boolean_return() {
            let engine = ScriptEngine::new().unwrap();
            let context = serde_json::json!({});
            let result = engine
                .execute_script("true", &context)
                .await
                .unwrap();
            assert!(result.success);
            assert_eq!(result.result, Some(serde_json::json!(true)));
        }
    
        #[tokio::test]
        async fn test_object_return() {
            let engine = ScriptEngine::new().unwrap();
            let context = serde_json::json!({});
            let result = engine
                .execute_script("({ a: 1, b: 'test' })", &context)
                .await
                .unwrap();
            assert!(result.success);
            assert_eq!(result.result, Some(serde_json::json!({ "a": 1.0, "b": "test" })));
        }
    
        #[tokio::test]
        async fn test_array_return() {
            let engine = ScriptEngine::new().unwrap();
            let context = serde_json::json!({});
            let result = engine
                .execute_script("[1, 'test', true]", &context)
                .await
                .unwrap();
            assert!(result.success);
            assert_eq!(result.result, Some(serde_json::json!([1.0, "test", true])));
        }

    #[tokio::test]
    async fn test_validation_script() {
        let engine = ScriptEngine::new().unwrap();
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let context = ValidationContext {
            status_code: 200,
            headers,
            body: r#"{"status": "ok"}"#.to_string(),
            response_time: 150,
        };

        let script = r#"
            assert(context.status_code === 200, "Status code should be 200");
            assert(context.response_time < 1000, "Response time should be less than 1000ms");
            
            const body = JSON.parse(context.body);
            assert(body.status === "ok", "Status should be ok");

            true
        "#;

        let result = engine
            .execute_validation_script(script, &context)
            .await
            .unwrap();

        assert!(result.passed);
        assert_eq!(result.details, Some(serde_json::json!(true)));
    }

    #[tokio::test]
    async fn test_failing_validation_script() {
        let engine = ScriptEngine::new().unwrap();
        let context = ValidationContext {
            status_code: 500,
            headers: HashMap::new(),
            body: "Error".to_string(),
            response_time: 2000,
        };

        let script = r#"
            assert(context.status_code === 200, "Status code should be 200");
        "#;

        let result = engine
            .execute_validation_script(script, &context)
            .await
            .unwrap();
        println!("Script result: {:?}", result);
        assert!(!result.passed);
        assert_eq!(result.message, "Status code should be 200");
    }
}
