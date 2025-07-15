use axum::{
    Router,
    extract::State,
    http::StatusCode,
    response::{Json, Response},
    routing::{get, post},
};
use monitor_core::{Error, auth::AuthService, cache::RedisPool, config::Config, db::DatabasePool};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: DatabasePool,
    pub redis: RedisPool,
    pub auth: AuthService,
    pub config: Config,
}

#[derive(Debug)]
pub struct ApiError(Error);

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        ApiError(err)
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self.0 {
            Error::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            Error::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Error::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

pub async fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/login", post(login))
        .route("/api/auth/register", post(register))
        .route("/api/monitors", get(get_monitors))
        .route("/api/monitors", post(create_monitor))
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        .with_state(state)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}

async fn login(State(_state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(json!({
        "message": "Login endpoint - TODO: implement"
    })))
}

async fn register(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(json!({
        "message": "Register endpoint - TODO: implement"
    })))
}

async fn get_monitors(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(json!({
        "monitors": [],
        "message": "Get monitors endpoint - TODO: implement"
    })))
}

async fn create_monitor(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(json!({
        "message": "Create monitor endpoint - TODO: implement"
    })))
}
