use monitor_core::{Error, Result};
/// 引擎核心模块
///
/// 提供JavaScript脚本执行环境，支持脚本验证、超时控制和错误处理
use rquickjs::{Context, Runtime, Value as JsValue, Ctx};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

use crate::models::{ScriptResult, SecurityConfig, ValidationContext, ValidationResult};

/// JavaScript脚本执行引擎
///
/// 基于rquickjs的JavaScript运行时，提供安全的脚本执行环境
/// 支持超时控制、错误处理和上下文数据传递
///
/// # 主要功能
/// - 执行任意JavaScript代码
/// - 提供验证脚本执行功能
/// - 支持超时控制防止无限循环
/// - 提供详细的错误信息和调试支持
/// - 内存和栈大小限制
/// - 函数黑名单安全控制
///
/// # 示例
/// ```
/// let engine = ScriptEngine::new().unwrap();
/// let result = engine.execute_script("1 + 1", &json!({})).await;
/// ```
pub struct ScriptEngine {
    /// JavaScript运行时实例
    runtime: Runtime,
    /// 脚本执行的最大超时时间
    timeout: Duration,
    /// 安全配置
    security_config: SecurityConfig,
}

impl ScriptEngine {
    /// 创建一个新的ScriptEngine实例，使用默认的30秒超时时间和默认安全配置
    ///
    /// # 返回值
    /// 返回一个新的ScriptEngine实例
    ///
    /// # 错误处理
    /// 如果创建Runtime失败，返回错误
    pub fn new() -> Result<Self> {
        Self::with_config(Duration::from_secs(30), SecurityConfig::default())
    }

    /// 使用指定超时时间创建ScriptEngine实例，使用默认安全配置
    ///
    /// # 参数
    /// * `timeout` - 脚本执行的最大允许时间
    ///
    /// # 返回值
    /// 返回一个新的ScriptEngine实例
    ///
    /// # 错误处理
    /// 如果创建Runtime失败，返回错误
    pub fn with_timeout(timeout: Duration) -> Result<Self> {
        Self::with_config(timeout, SecurityConfig::default())
    }

    /// 使用指定的安全配置创建ScriptEngine实例
    ///
    /// # 参数
    /// * `security_config` - 安全配置
    ///
    /// # 返回值
    /// 返回一个新的ScriptEngine实例
    ///
    /// # 错误处理
    /// 如果创建Runtime失败，返回错误
    pub fn with_security_config(security_config: SecurityConfig) -> Result<Self> {
        Self::with_config(Duration::from_secs(30), security_config)
    }

    /// 使用指定超时时间和安全配置创建ScriptEngine实例
    ///
    /// # 参数
    /// * `timeout` - 脚本执行的最大允许时间
    /// * `security_config` - 安全配置
    ///
    /// # 返回值
    /// 返回一个新的ScriptEngine实例
    ///
    /// # 错误处理
    /// 如果创建Runtime失败，返回错误
    pub fn with_config(timeout: Duration, security_config: SecurityConfig) -> Result<Self> {
        
        // 创建带有内存和栈限制的运行时
        let runtime = Runtime::new()
            .map_err(|e| Error::script_execution(format!("Failed to create runtime: {}", e)))?;
        
        // 设置内存限制和栈大小限制
        runtime.set_memory_limit(security_config.memory_limit);
        runtime.set_max_stack_size(security_config.stack_size);

        Ok(Self {
            runtime,
            timeout,
            security_config,
        })
    }

    /// 执行给定的JavaScript脚本并返回结果
    ///
    /// # 参数
    /// * `script` - 要执行的JavaScript代码
    /// * `context_data` - 传递给脚本的上下文数据
    ///
    /// # 返回值
    /// 返回包含执行结果或错误信息的ScriptResult
    ///
    /// # 实现逻辑
    /// 1. 创建JavaScript执行上下文
    /// 2. 设置上下文数据和工具函数
    /// 3. 执行脚本并记录执行时间
    /// 4. 处理执行结果（成功或失败）
    pub async fn execute_script(&self, script: &str, context_data: &Value) -> Result<ScriptResult> {
        let start_time = Instant::now();
        let script_with_metadata = self.wrap_script_with_metadata(script);

        let ctx = Context::full(&self.runtime)
            .map_err(|e| Error::script_execution(format!("Failed to create context: {}", e)))?;

        let result: Result<ScriptResult> = ctx.with(|ctx| {
            // Set up the context with monitor data
            let global = ctx.globals();

            // 应用安全策略 - 禁用危险函数
            if let Err(e) = self.apply_security_policies(&ctx) {
                return Err(Error::script_execution(format!(
                    "Failed to apply security policies: {}",
                    e
                )));
            }

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

    /// 创建带有元数据的脚本包装器，用于增强错误报告和超时处理
    ///
    /// # 参数
    /// * `script` - 原始JavaScript代码
    ///
    /// # 返回值
    /// 返回包装后的JavaScript代码
    ///
    /// # 实现逻辑
    /// 1. 对于简单表达式不进行包装
    /// 2. 对于复杂脚本添加超时检查和错误处理
    /// 3. 返回包装后的脚本代码
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

        // 从外部文件加载脚本包装器模板
        let wrapper_template = include_str!("script_wrapper.js");

        // 将用户脚本插入到包装器模板中
        wrapper_template.replace("{script}", script)
    }

    /// 获取工具函数的JavaScript代码
    ///
    /// # 返回值
    /// 返回包含工具函数的字符串
    ///
    /// # 实现逻辑
    /// 从外部文件加载工具函数
    fn get_utility_functions(&self) -> String {
        // Load utility functions from an external file
        let utility_script = include_str!("utility_functions.js");
        utility_script.to_string()
    }

    /// 提取详细的错误信息
    ///
    /// # 参数
    /// * `error` - JavaScript错误对象
    /// * `original_script` - 原始脚本代码
    ///
    /// # 返回值
    /// 返回包含详细错误信息的JSON对象
    ///
    /// # 实现逻辑
    /// 1. 处理异常类型错误
    /// 2. 提取错误消息
    /// 3. 获取脚本预览
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

    /// 解析错误消息并生成详细的错误信息
    ///
    /// # 参数
    /// * `error_msg` - 错误消息字符串
    /// * `script` - 原始脚本代码
    ///
    /// # 返回值
    /// 返回包含详细错误信息的JSON对象，如果无法解析则返回None
    ///
    /// # 实现逻辑
    /// 1. 检查错误类型（语法错误、引用错误、类型错误）
    /// 2. 生成相应的错误信息和建议
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

    /// 获取脚本预览
    ///
    /// # 参数
    /// * `script` - 原始脚本代码
    /// * `error_line` - 错误发生的行号（可选）
    ///
    /// # 返回值
    /// 返回包含脚本预览信息的JSON对象
    ///
    /// # 实现逻辑
    /// 1. 如果有错误行号，显示该行附近的代码
    /// 2. 否则显示脚本开头的若干行
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

    /// 应用安全策略到JavaScript上下文
    ///
    /// # 参数
    /// * `ctx` - JavaScript执行上下文
    ///
    /// # 返回值
    /// 如果成功应用安全策略返回Ok(())，否则返回错误
    ///
    /// # 实现逻辑
    /// 1. 禁用危险的全局函数
    /// 2. 根据配置禁用eval和Function构造函数
    /// 3. 设置安全的全局对象
    fn apply_security_policies(&self, ctx: &Ctx) -> Result<()> {
        let _global = ctx.globals();

        // 禁用配置中指定的危险函数
        for func_name in &self.security_config.denied_functions {
            // 将危险函数设置为undefined或抛出错误的函数
            let error_message = format!("Access to '{}' is denied for security reasons", func_name);
            let deny_script = format!(
                r#"
                (function() {{
                    const originalFunc = globalThis['{}'];
                    globalThis['{}'] = function() {{
                        throw new Error('{}');
                    }};
                    // 也尝试在window对象上禁用（如果存在）
                    if (typeof window !== 'undefined') {{
                        window['{}'] = globalThis['{}'];
                    }}
                    // 尝试删除属性
                    try {{
                        delete globalThis['{}'];
                    }} catch(e) {{
                        // 如果无法删除，至少覆盖它
                    }}
                }})();
                "#,
                func_name, func_name, error_message, func_name, func_name, func_name
            );

            ctx.eval::<(), _>(deny_script)
                .map_err(|e| Error::script_execution(format!("Failed to deny function {}: {}", func_name, e)))?;
        }

        // 特殊处理eval函数
        if self.security_config.disable_eval {
            let eval_deny_script = r#"
                (function() {
                    const originalEval = globalThis.eval;
                    globalThis.eval = function() {
                        throw new Error('eval() is disabled for security reasons');
                    };
                    // 也禁用间接eval
                    try {
                        Object.defineProperty(globalThis, 'eval', {
                            value: function() {
                                throw new Error('eval() is disabled for security reasons');
                            },
                            writable: false,
                            configurable: false
                        });
                    } catch(e) {
                        // 如果无法重新定义，至少覆盖它
                    }
                })();
            "#;

            ctx.eval::<(), _>(eval_deny_script)
                .map_err(|e| Error::script_execution(format!("Failed to disable eval: {}", e)))?;
        }

        // 特殊处理Function构造函数
        if self.security_config.disable_function_constructor {
            let function_deny_script = r#"
                (function() {
                    const originalFunction = globalThis.Function;
                    globalThis.Function = function() {
                        throw new Error('Function constructor is disabled for security reasons');
                    };
                    try {
                        Object.defineProperty(globalThis, 'Function', {
                            value: function() {
                                throw new Error('Function constructor is disabled for security reasons');
                            },
                            writable: false,
                            configurable: false
                        });
                    } catch(e) {
                        // 如果无法重新定义，至少覆盖它
                    }
                })();
            "#;

            ctx.eval::<(), _>(function_deny_script)
                .map_err(|e| Error::script_execution(format!("Failed to disable Function constructor: {}", e)))?;
        }

        // 禁用模块导入
        if self.security_config.disable_modules {
            let module_deny_script = r#"
                (function() {
                    // 禁用动态import
                    if (typeof globalThis.import !== 'undefined') {
                        globalThis.import = function() {
                            throw new Error('Dynamic imports are disabled for security reasons');
                        };
                    }
                    
                    // 禁用require（如果存在）
                    if (typeof globalThis.require !== 'undefined') {
                        globalThis.require = function() {
                            throw new Error('require() is disabled for security reasons');
                        };
                    }
                })();
            "#;

            ctx.eval::<(), _>(module_deny_script)
                .map_err(|e| Error::script_execution(format!("Failed to disable modules: {}", e)))?;
        }

        // 添加安全监控函数
        let security_monitor_script = r#"
            (function() {
                // 监控内存使用情况的辅助函数
                globalThis.__checkMemory = function() {
                    // 这里可以添加内存检查逻辑
                    // QuickJS会自动处理内存限制
                    return true;
                };
                
                // 监控执行时间的辅助函数
                globalThis.__checkTimeout = function() {
                    if (typeof globalThis.__start_time !== 'undefined' && 
                        typeof globalThis.__timeout_ms !== 'undefined') {
                        const elapsed = Date.now() - globalThis.__start_time;
                        if (elapsed > globalThis.__timeout_ms) {
                            throw new Error('Script execution timeout exceeded');
                        }
                    }
                    return true;
                };
            })();
        "#;

        ctx.eval::<(), _>(security_monitor_script)
            .map_err(|e| Error::script_execution(format!("Failed to setup security monitoring: {}", e)))?;

        Ok(())
    }

    /// 获取当前的安全配置
    ///
    /// # 返回值
    /// 返回当前使用的安全配置的克隆
    pub fn get_security_config(&self) -> SecurityConfig {
        self.security_config.clone()
    }

    /// 获取当前运行时的内存使用情况
    ///
    /// # 返回值
    /// 返回内存使用情况（字节），如果无法获取则返回None
    ///
    /// # 注意
    /// 这个功能依赖于QuickJS的内存统计功能
    pub fn get_memory_usage(&self) -> Option<usize> {
        // QuickJS的rquickjs绑定可能不直接暴露内存使用情况
        // 这里返回None，但可以在未来版本中实现
        None
    }

    /// 执行验证脚本
    ///
    /// # 参数
    /// * `script` - 验证脚本代码
    /// * `response_data` - 传递给脚本的响应数据
    ///
    /// # 返回值
    /// 返回包含验证结果的ValidationResult
    ///
    /// # 实现逻辑
    /// 1. 将响应数据序列化为JSON
    /// 2. 执行验证脚本
    /// 3. 根据执行结果生成验证结果
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
                .clone()
                .map(|v| match v {
                    Value::Bool(b) => b,
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

/// 将JavaScript值转换为Rust的serde_json::Value
///
/// # 参数
/// * `value` - 要转换的JavaScript值（rquickjs::Value）
///
/// # 返回值
/// 返回转换后的serde_json::Value，如果转换失败则返回错误
///
/// # 处理逻辑
/// 1. 处理基本类型：undefined、null、布尔值、数字、字符串
/// 2. 处理复杂类型：数组、函数、对象、符号
/// 3. 处理特殊对象：Date、RegExp、Error
/// 4. 为未知类型提供回退处理
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

/// ScriptEngine的默认实现
///
/// 使用30秒超时时间创建一个新的ScriptEngine实例
///
/// # 返回值
/// 返回一个新的ScriptEngine实例
///
/// # 注意
/// 如果创建Runtime失败，此实现会panic
impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default ScriptEngine")
    }
}
