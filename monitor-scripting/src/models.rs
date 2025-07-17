use std::collections::HashSet;

use serde_json::Value;

/// 默认内存限制 (8MB)
pub const DEFAULT_MEMORY_LIMIT: usize = 8 * 1024 * 1024;
/// 默认栈大小限制 (512KB)
pub const DEFAULT_STACK_SIZE: usize = 512 * 1024;

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

/// 安全配置结构体
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// 内存限制（字节）
    pub memory_limit: usize,
    /// 栈大小限制（字节）
    pub stack_size: usize,
    /// 禁用的全局函数列表
    pub denied_functions: HashSet<String>,
    /// 禁用的全局对象属性列表
    pub denied_properties: HashSet<String>,
    /// 是否禁用eval函数
    pub disable_eval: bool,
    /// 是否禁用Function构造函数
    pub disable_function_constructor: bool,
    /// 是否禁用模块导入
    pub disable_modules: bool,
    /// 是否启用严格模式
    pub enable_strict_mode: bool,
    /// 最大循环迭代次数限制
    pub max_loop_iterations: Option<u64>,
    /// 最大递归深度限制
    pub max_recursion_depth: Option<u32>,
    /// 是否禁用原型链修改
    pub disable_prototype_pollution: bool,
    /// 是否启用内存使用监控
    pub enable_memory_monitoring: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut denied_functions = HashSet::new();
        // 默认禁用的危险函数
        denied_functions.insert("eval".to_string());
        denied_functions.insert("Function".to_string());
        denied_functions.insert("setTimeout".to_string());
        denied_functions.insert("setInterval".to_string());
        denied_functions.insert("setImmediate".to_string());
        denied_functions.insert("require".to_string());
        denied_functions.insert("import".to_string());
        denied_functions.insert("importScripts".to_string());
        denied_functions.insert("XMLHttpRequest".to_string());
        denied_functions.insert("fetch".to_string());
        denied_functions.insert("WebSocket".to_string());
        denied_functions.insert("Worker".to_string());
        denied_functions.insert("SharedWorker".to_string());
        denied_functions.insert("ServiceWorker".to_string());

        let mut denied_properties = HashSet::new();
        // 默认禁用的危险属性
        denied_properties.insert("constructor".to_string());
        denied_properties.insert("__proto__".to_string());
        denied_properties.insert("prototype".to_string());

        Self {
            memory_limit: DEFAULT_MEMORY_LIMIT,
            stack_size: DEFAULT_STACK_SIZE,
            denied_functions,
            disable_eval: true,
            disable_function_constructor: true,
            disable_modules: true,
            denied_properties,
            enable_strict_mode: true,
            max_loop_iterations: Some(10000),
            max_recursion_depth: Some(100),
            disable_prototype_pollution: true,
            enable_memory_monitoring: true,
        }
    }
}

impl SecurityConfig {
    /// 创建一个宽松的安全配置（用于测试或受信任的环境）
    pub fn permissive() -> Self {
        let mut denied_properties = HashSet::new();
        // 宽松模式下只禁用最基本的危险属性
        denied_properties.insert("__proto__".to_string());

        Self {
            memory_limit: DEFAULT_MEMORY_LIMIT * 4, // 32MB
            stack_size: DEFAULT_STACK_SIZE * 4,     // 2MB
            denied_functions: HashSet::new(),
            disable_eval: false,
            disable_function_constructor: false,
            disable_modules: false,
            denied_properties,
            enable_strict_mode: false,
            max_loop_iterations: Some(100000),
            max_recursion_depth: Some(1000),
            disable_prototype_pollution: false,
            enable_memory_monitoring: false,
        }
    }

    /// 创建一个严格的安全配置（用于生产环境）
    pub fn strict() -> Self {
        let mut denied_functions = HashSet::new();
        // 严格模式下禁用更多函数
        denied_functions.insert("eval".to_string());
        denied_functions.insert("Function".to_string());
        denied_functions.insert("setTimeout".to_string());
        denied_functions.insert("setInterval".to_string());
        denied_functions.insert("setImmediate".to_string());
        denied_functions.insert("require".to_string());
        denied_functions.insert("import".to_string());
        denied_functions.insert("importScripts".to_string());
        denied_functions.insert("XMLHttpRequest".to_string());
        denied_functions.insert("fetch".to_string());
        denied_functions.insert("WebSocket".to_string());
        denied_functions.insert("Worker".to_string());
        denied_functions.insert("SharedWorker".to_string());
        denied_functions.insert("ServiceWorker".to_string());
        denied_functions.insert("localStorage".to_string());
        denied_functions.insert("sessionStorage".to_string());
        denied_functions.insert("indexedDB".to_string());
        denied_functions.insert("webkitStorageInfo".to_string());
        denied_functions.insert("navigator".to_string());
        denied_functions.insert("location".to_string());
        denied_functions.insert("history".to_string());
        denied_functions.insert("document".to_string());
        denied_functions.insert("window".to_string());
        denied_functions.insert("global".to_string());
        denied_functions.insert("globalThis".to_string());
        denied_functions.insert("process".to_string());
        denied_functions.insert("Buffer".to_string());

        let mut denied_properties = HashSet::new();
        // 严格模式下禁用更多属性
        denied_properties.insert("constructor".to_string());
        denied_properties.insert("__proto__".to_string());
        denied_properties.insert("prototype".to_string());
        denied_properties.insert("caller".to_string());
        denied_properties.insert("callee".to_string());
        denied_properties.insert("arguments".to_string());

        Self {
            memory_limit: DEFAULT_MEMORY_LIMIT / 2, // 4MB
            stack_size: DEFAULT_STACK_SIZE / 2,     // 256KB
            denied_functions,
            disable_eval: true,
            disable_function_constructor: true,
            disable_modules: true,
            denied_properties,
            enable_strict_mode: true,
            max_loop_iterations: Some(1000),
            max_recursion_depth: Some(50),
            disable_prototype_pollution: true,
            enable_memory_monitoring: true,
        }
    }

    /// 添加禁用函数
    pub fn deny_function(&mut self, function_name: &str) -> &mut Self {
        self.denied_functions.insert(function_name.to_string());
        self
    }

    /// 移除禁用函数
    pub fn allow_function(&mut self, function_name: &str) -> &mut Self {
        self.denied_functions.remove(function_name);
        self
    }

    /// 设置内存限制
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = limit;
        self
    }

    /// 设置栈大小限制
    pub fn with_stack_size(mut self, size: usize) -> Self {
        self.stack_size = size;
        self
    }
}
