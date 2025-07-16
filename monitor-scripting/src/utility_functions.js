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