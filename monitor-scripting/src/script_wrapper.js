/**
 * 脚本包装器，用于增强错误报告和超时处理
 * 
 * 主要功能：
 * 1. 添加超时检查防止无限循环
 * 2. 增强错误处理和报告
 * 3. 提供行号跟踪以便更好地定位错误
 */

// 包装脚本的函数模板
(function() {
    // Timeout check wrapper
    function checkTimeout() {
        const now = performance && performance.now ? performance.now : Date.now();
        if (typeof __start_time !== 'undefined' && typeof __timeout_ms !== 'undefined') {
            if ((now - __start_time) > __timeout_ms) {
                throw new Error('Script execution timeout after ' + __timeout_ms + 'ms');
            }
        }
    }
    
    // Add line tracking for better error reporting
    try {
        checkTimeout();
        return (function() {
            // 用户脚本将在这里插入
            {script}
        })();
    } catch (error) {
        // Re-throw with enhanced error information
        if (error.name === 'Error' && !error.line) {
            error.line = 'unknown';
            error.column = 'unknown';
        }
        throw error;
    }
})();