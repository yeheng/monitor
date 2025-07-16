use monitor_core::{Result, logging};
use crate::models::ValidationContext;
use std::collections::HashMap;
use tracing::info;

pub mod engine;
pub mod models;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init_logging();

    info!("🚀 Monitor Scripting Engine Demo - Enhanced Version");

    let script_engine = engine::ScriptEngine::new()?;

    // Demo 1: Basic script execution with different return types
    info!("📋 Demo 1: Testing different JavaScript return types");

    let demos = vec![
        ("Number", "42"),
        ("String", "'Hello World'"),
        ("Boolean", "true"),
        ("Array", "[1, 2, 'test', true]"),
        (
            "Object",
            "({ name: 'Monitor', version: 1.0, active: true })",
        ),
        ("Undefined", "undefined"),
        ("Function", "function test() { return 'works'; }; test"),
        ("Date", "new Date('2024-01-01')"),
        ("Null", "null"),
    ];

    for (name, script) in demos {
        let context = serde_json::json!({});
        match script_engine.execute_script(script, &context).await {
            Ok(result) => {
                info!(
                    "  ✅ {}: {:?} ({}ms)",
                    name, result.result, result.execution_time_ms
                );
            }
            Err(e) => {
                info!("  ❌ {}: {}", name, e);
            }
        }
    }

    // Demo 2: Enhanced validation script with utilities
    info!("📋 Demo 2: Enhanced validation script with HTTP response");

    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("x-response-time".to_string(), "150ms".to_string());

    let validation_context = ValidationContext {
        status_code: 200,
        headers,
        body: r#"{"status": "success", "data": {"users": 42, "active": true}, "timestamp": "2024-01-01T00:00:00Z"}"#.to_string(),
        response_time: 150,
    };

    let enhanced_validation_script = r#"
        info('Starting enhanced validation script');
        
        // HTTP status validation
        assertStatus(context.status_code, 200);
        assertStatusRange(context.status_code, 200, 299);
        info('✅ HTTP status checks passed');
        
        // Content validation
        assertContains(context.body, 'success');
        assertMatches(context.body, /users.*\d+/);
        info('✅ Content validation passed');
        
        // JSON structure validation  
        const body = parseJSON(context.body);
        assertValidJSON(context.body);
        expect(body.status, 'success');
        assertType(body.data.users, 'number');
        expect(body.data.active, true);
        info('✅ JSON structure validation passed');
        
        // Performance validation
        assert(context.response_time < 1000, 'Response time should be under 1s');
        info(`✅ Response time: ${context.response_time}ms`);
        
        // Return comprehensive validation result
        ({
            validation_summary: {
                http_status: 'pass',
                content_format: 'pass', 
                json_structure: 'pass',
                performance: 'pass'
            },
            extracted_data: {
                user_count: body.data.users,
                is_active: body.data.active,
                response_time_ms: context.response_time
            },
            recommendations: body.data.users > 50 ? ['Consider pagination'] : ['Good user count']
        })
    "#;

    match script_engine
        .execute_validation_script(enhanced_validation_script, &validation_context)
        .await
    {
        Ok(result) => {
            info!("  ✅ Validation passed: {}", result.passed);
            info!("  📊 Execution time: {}ms", result.execution_time_ms);
            if let Some(details) = result.details {
                info!(
                    "  📋 Validation details: {}",
                    serde_json::to_string_pretty(&details).unwrap_or_default()
                );
            }
        }
        Err(e) => {
            info!("  ❌ Validation failed: {}", e);
        }
    }

    // Demo 3: Error handling and debugging
    info!("📋 Demo 3: Error handling and detailed debugging");

    let error_scripts = vec![
        ("Syntax Error", "function test( { // missing )"),
        ("Reference Error", "console.log(undefinedVariable)"),
        ("Type Error", "null.someMethod()"),
        (
            "Custom Error",
            "throw new Error('Custom validation failed: Expected status 200, got 404')",
        ),
        (
            "Assertion Error",
            "assert(false, 'This should fail for demo purposes')",
        ),
    ];

    for (name, script) in error_scripts {
        let context = serde_json::json!({});
        match script_engine.execute_script(script, &context).await {
            Ok(result) => {
                if !result.success {
                    info!("  🔍 {}: Error captured", name);
                    if let Some(error) = result.error {
                        info!(
                            "     Type: {}",
                            error
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                        );
                        info!(
                            "     Message: {}",
                            error
                                .get("message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("no message")
                        );
                        if let Some(suggestion) = error.get("suggestion") {
                            info!("     💡 Suggestion: {}", suggestion.as_str().unwrap_or(""));
                        }
                    }
                }
            }
            Err(e) => {
                info!("  ❌ {}: {}", name, e);
            }
        }
    }

    // Demo 4: Performance timing utilities
    info!("📋 Demo 4: Performance timing and monitoring");

    let performance_script = r#"
        info('Testing performance timing utilities');
        
        const totalTimer = time('total-operation');
        
        // Simulate database query
        const dbTimer = time('database-query');
        for (let i = 0; i < 1000; i++) {
            Math.sqrt(i * Math.random());
        }
        const dbTime = dbTimer.end();
        
        // Simulate API call
        const apiTimer = time('api-call');
        for (let i = 0; i < 500; i++) {
            JSON.stringify({ id: i, data: Math.random() });
        }
        const apiTime = apiTimer.end();
        
        const totalTime = totalTimer.end();
        
        ({
            performance_metrics: {
                database_time_ms: dbTime,
                api_time_ms: apiTime,
                total_time_ms: totalTime
            },
            performance_grade: totalTime < 100 ? 'excellent' : totalTime < 500 ? 'good' : 'needs_optimization'
        })
    "#;

    let context = serde_json::json!({});
    match script_engine
        .execute_script(performance_script, &context)
        .await
    {
        Ok(result) => {
            info!(
                "  ⏱️ Performance test completed in {}ms",
                result.execution_time_ms
            );
            if let Some(details) = result.result {
                info!(
                    "  📊 Performance metrics: {}",
                    serde_json::to_string_pretty(&details).unwrap_or_default()
                );
            }
        }
        Err(e) => {
            info!("  ❌ Performance test failed: {}", e);
        }
    }

    info!("🎉 All demos completed! The enhanced scripting engine supports:");
    info!("   • Multiple JavaScript return types (objects, arrays, functions, etc.)");
    info!("   • Detailed error reporting with suggestions and script previews");
    info!("   • Enhanced validation utilities for HTTP responses");
    info!("   • Performance timing and execution metrics");
    info!("   • Timeout handling and resource management");
    info!("   • Comprehensive debugging information");

    Ok(())
}
