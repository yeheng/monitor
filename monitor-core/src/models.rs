use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Monitor {
    pub id: Uuid,
    pub name: String,
    pub endpoint: String,
    pub method: String,
    pub headers: Option<serde_json::Value>,
    pub body: Option<String>,
    pub expected_status: i32,
    pub timeout: i32,
    pub interval: i32,
    pub script: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MonitorResult {
    pub id: Uuid,
    pub monitor_id: Uuid,
    pub status: String,
    pub response_time: i32,
    pub response_code: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Alert {
    pub id: Uuid,
    pub monitor_id: Uuid,
    pub type_: String,
    pub config: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMonitorRequest {
    pub name: String,
    pub endpoint: String,
    pub method: String,
    pub headers: Option<serde_json::Value>,
    pub body: Option<String>,
    pub expected_status: i32,
    pub timeout: i32,
    pub interval: i32,
    pub script: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMonitorRequest {
    pub name: Option<String>,
    pub endpoint: Option<String>,
    pub method: Option<String>,
    pub headers: Option<serde_json::Value>,
    pub body: Option<String>,
    pub expected_status: Option<i32>,
    pub timeout: Option<i32>,
    pub interval: Option<i32>,
    pub script: Option<String>,
    pub enabled: Option<bool>,
}