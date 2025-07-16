// 增强的日志记录功能，支持不同级别
/**
 * 记录日志消息
 * @param {string} message - 要记录的消息内容
 * @param {string} level - 日志级别，默认为 'INFO'
 * 输出：在控制台打印带时间戳和级别的格式化日志
 * 逻辑：获取当前时间戳，格式化输出日志信息
 */
function log(message, level = "INFO") {
  const timestamp = new Date().toISOString();
  console.log(`[${timestamp}] [${level}] [Script] ${message}`);
}

/**
 * 记录调试级别日志
 * @param {string} message - 调试消息
 * 输出：DEBUG级别的日志
 */
function debug(message) {
  log(message, "DEBUG");
}

/**
 * 记录信息级别日志
 * @param {string} message - 信息消息
 * 输出：INFO级别的日志
 */
function info(message) {
  log(message, "INFO");
}

/**
 * 记录警告级别日志
 * @param {string} message - 警告消息
 * 输出：WARN级别的日志
 */
function warn(message) {
  log(message, "WARN");
}

/**
 * 记录错误级别日志
 * @param {string} message - 错误消息
 * 输出：ERROR级别的日志
 */
function error(message) {
  log(message, "ERROR");
}

// 增强的断言函数
/**
 * 断言条件为真
 * @param {boolean} condition - 要检查的条件
 * @param {string} message - 可选的错误消息
 * 输出：如果条件为假则抛出AssertionError，否则返回true
 * 逻辑：检查条件是否为真，如果为假则创建并抛出断言错误
 */
function assert(condition, message) {
  if (!condition) {
    const error = new Error(message || "Assertion failed");
    error.name = "AssertionError";
    throw error;
  }
  return true;
}

/**
 * 期望值匹配检查
 * @param {any} actual - 实际值
 * @param {any} expected - 期望值
 * @param {string} message - 可选的错误消息
 * 输出：如果值不匹配则抛出ExpectationError，否则返回true
 * 逻辑：比较实际值和期望值，不相等时抛出包含详细信息的错误
 */
function expect(actual, expected, message) {
  if (actual !== expected) {
    const error = new Error(
      message ||
        `Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`
    );
    error.name = "ExpectationError";
    error.actual = actual;
    error.expected = expected;
    throw error;
  }
  return true;
}

// 类型检查工具函数
/**
 * 断言值的类型
 * @param {any} value - 要检查的值
 * @param {string} expectedType - 期望的类型名称
 * @param {string} message - 可选的错误消息
 * 输出：如果类型不匹配则抛出错误，否则返回true
 * 逻辑：使用typeof检查值的类型，与期望类型比较
 */
function assertType(value, expectedType, message) {
  const actualType = typeof value;
  if (actualType !== expectedType) {
    throw new Error(
      message || `Expected type ${expectedType}, got ${actualType}`
    );
  }
  return true;
}

/**
 * 断言值是指定构造函数的实例
 * @param {any} value - 要检查的值
 * @param {Function} constructor - 期望的构造函数
 * @param {string} message - 可选的错误消息
 * 输出：如果不是指定实例则抛出错误，否则返回true
 * 逻辑：使用instanceof操作符检查值是否为指定构造函数的实例
 */
function assertInstanceOf(value, constructor, message) {
  if (!(value instanceof constructor)) {
    throw new Error(
      message || `Expected instance of ${constructor.name}, got ${typeof value}`
    );
  }
  return true;
}

// HTTP响应验证工具函数
/**
 * 断言HTTP状态码
 * @param {number} statusCode - 实际的状态码
 * @param {number} expected - 期望的状态码
 * @param {string} message - 可选的错误消息
 * 输出：如果状态码不匹配则抛出错误，否则返回true
 * 逻辑：调用expect函数比较实际状态码和期望状态码
 */
function assertStatus(statusCode, expected, message) {
  return expect(
    statusCode,
    expected,
    message || `Expected status ${expected}, got ${statusCode}`
  );
}

/**
 * 断言HTTP状态码在指定范围内
 * @param {number} statusCode - 要检查的状态码
 * @param {number} min - 最小值（包含）
 * @param {number} max - 最大值（包含）
 * @param {string} message - 可选的错误消息
 * 输出：如果状态码不在范围内则抛出错误，否则返回true
 * 逻辑：检查状态码是否在min和max之间（包含边界值）
 */
function assertStatusRange(statusCode, min, max, message) {
  if (statusCode < min || statusCode > max) {
    throw new Error(
      message || `Expected status between ${min}-${max}, got ${statusCode}`
    );
  }
  return true;
}

/**
 * 断言文本包含指定子字符串
 * @param {string} text - 要检查的文本
 * @param {string} substring - 期望包含的子字符串
 * @param {string} message - 可选的错误消息
 * 输出：如果文本不包含子字符串则抛出错误，否则返回true
 * 逻辑：检查text是否为字符串类型且包含指定的substring
 */
function assertContains(text, substring, message) {
  if (typeof text !== "string" || !text.includes(substring)) {
    throw new Error(message || `Expected text to contain "${substring}"`);
  }
  return true;
}

/**
 * 断言文本匹配指定的正则表达式模式
 * @param {string} text - 要检查的文本
 * @param {RegExp|string} pattern - 正则表达式模式
 * @param {string} message - 可选的错误消息
 * 输出：如果文本不匹配模式则抛出错误，否则返回true
 * 逻辑：将pattern转换为RegExp对象（如果不是），然后测试text是否匹配
 */
function assertMatches(text, pattern, message) {
  const regex = pattern instanceof RegExp ? pattern : new RegExp(pattern);
  if (!regex.test(text)) {
    throw new Error(message || `Expected text to match pattern ${regex}`);
  }
  return true;
}

// JSON处理工具函数，带错误处理
/**
 * 解析JSON文本
 * @param {string} text - 要解析的JSON字符串
 * @param {any} defaultValue - 解析失败时的默认值，默认为null
 * 输出：解析成功返回JavaScript对象，失败时返回默认值或抛出错误
 * 逻辑：尝试解析JSON，如果失败且提供了默认值则返回默认值，否则抛出详细错误
 */
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

/**
 * 断言文本是有效的JSON格式
 * @param {string} text - 要验证的JSON字符串
 * @param {string} message - 可选的错误消息
 * 输出：如果JSON无效则抛出错误，否则返回true
 * 逻辑：尝试解析JSON字符串，如果解析失败则抛出包含详细信息的错误
 */
function assertValidJSON(text, message) {
  try {
    JSON.parse(text);
    return true;
  } catch (e) {
    throw new Error(message || `Invalid JSON: ${e.message}`);
  }
}

// 性能计时工具函数
const performance = globalThis.performance || {
  now: function () {
    return Date.now();
  },
};

/**
 * 创建性能计时器
 * @param {string} label - 计时器标签名称
 * 输出：返回包含end方法的计时器对象
 * 逻辑：记录开始时间，返回对象包含end方法用于计算和记录执行时长
 */
function time(label) {
  const start = performance.now();
  return {
    /**
     * 结束计时并记录结果
     * 输出：返回执行时长（毫秒），同时记录到日志
     * 逻辑：计算当前时间与开始时间的差值，格式化为毫秒并记录日志
     */
    end: function () {
      const duration = performance.now() - start;
      log(`${label}: ${duration.toFixed(2)}ms`, "TIMER");
      return duration;
    },
  };
}
