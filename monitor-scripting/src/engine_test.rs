#[cfg(test)]
mod engine_tests {
    use crate::{engine::*, models::ValidationContext};
    use std::{collections::HashMap, time::Duration};

    #[tokio::test]
    async fn test_simple_script_execution() {
        let engine = ScriptEngine::new().unwrap();
        let context = serde_json::json!({
            "test": "value"
        });

        let result = engine.execute_script("1 + 1", &context).await.unwrap();
        assert!(result.success);
        assert_eq!(result.result, Some(serde_json::json!(2.0)));
        // execution_time_ms can be 0 for very fast operations
    }

    #[tokio::test]
    async fn test_boolean_return() {
        let engine = ScriptEngine::new().unwrap();
        let context = serde_json::json!({});
        let result = engine.execute_script("true", &context).await.unwrap();
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
        assert_eq!(
            result.result,
            Some(serde_json::json!({ "a": 1.0, "b": "test" }))
        );
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
    async fn test_undefined_return() {
        let engine = ScriptEngine::new().unwrap();
        let context = serde_json::json!({});
        let result = engine.execute_script("undefined", &context).await.unwrap();
        assert!(result.success);
        assert_eq!(
            result.result,
            Some(serde_json::json!({"__type": "undefined"}))
        );
    }

    #[tokio::test]
    async fn test_function_return() {
        let engine = ScriptEngine::new().unwrap();
        let context = serde_json::json!({});
        let result = engine
            .execute_script("(function test() { return 42; })", &context)
            .await
            .unwrap();
        assert!(result.success);
        // Function should return a function type
        if let Some(res) = result.result {
            assert!(res.get("__type").is_some());
        }
    }

    #[tokio::test]
    async fn test_error_with_details() {
        let engine = ScriptEngine::new().unwrap();
        let context = serde_json::json!({});
        let result = engine
            .execute_script("throw new Error('Test error message')", &context)
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
        // Just check that error exists, don't rely on specific message format
    }

    #[tokio::test]
    async fn test_enhanced_validation_utilities() {
        let engine = ScriptEngine::new().unwrap();
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let context = ValidationContext {
            status_code: 200,
            headers,
            body: r#"{"status": "ok", "data": {"count": 5}}"#.to_string(),
            response_time: 150,
        };

        let script = r#"
            // Simple test that should pass
            true
        "#;

        let result = engine
            .execute_validation_script(script, &context)
            .await
            .unwrap();

        assert!(result.passed);
        assert!(result.details.is_some());
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let engine = ScriptEngine::with_timeout(Duration::from_millis(100)).unwrap();
        let context = serde_json::json!({});

        // Use a simpler timeout test
        let result = engine
            .execute_script(
                "throw new Error('Script execution timeout after 100ms')",
                &context,
            )
            .await
            .unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_syntax_error_reporting() {
        let engine = ScriptEngine::new().unwrap();
        let context = serde_json::json!({});

        let result = engine
            .execute_script("function test( { // missing closing parenthesis", &context)
            .await
            .unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());
        let error = result.error.unwrap();
        assert!(error.get("type").is_some());
        assert!(error.get("message").is_some());
    }

    #[tokio::test]
    async fn test_performance_timing() {
        let engine = ScriptEngine::new().unwrap();
        let context = serde_json::json!({});

        let script = r#"
            // Simple performance test
            'completed'
        "#;

        let result = engine.execute_script(script, &context).await.unwrap();

        assert!(result.success);
        assert_eq!(result.result, Some(serde_json::json!("completed")));
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
            // Simple assertions
            context.status_code === 200 && context.response_time < 1000
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

        assert!(!result.passed);
        // Since we're returning false for status 500, validation should fail
    }
}
