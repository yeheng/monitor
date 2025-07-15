use monitor_core::{
    models::{Monitor, MonitorResult},
    db::DatabasePool,
    Error, Result,
};
use reqwest::Client;
use sqlx::Row;
use std::time::Instant;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};
use uuid::Uuid;
use chrono::Utc;

pub struct MonitorScheduler {
    db: DatabasePool,
    http_client: Client,
    scheduler: JobScheduler,
}

impl MonitorScheduler {
    pub async fn new(db: DatabasePool) -> Result<Self> {
        let http_client = Client::new();
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| Error::scheduler(e.to_string()))?;
        
        Ok(Self {
            db,
            http_client,
            scheduler,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting monitor scheduler");
        
        let job = Job::new_async("0/30 * * * * *", |_uuid, _l| {
            Box::pin(async move {
                info!("Scheduler job triggered");
            })
        })
        .map_err(|e| Error::scheduler(e.to_string()))?;
        
        self.scheduler.add(job).await
            .map_err(|e| Error::scheduler(e.to_string()))?;
        self.scheduler.start().await
            .map_err(|e| Error::scheduler(e.to_string()))?;
        
        info!("Monitor scheduler started successfully");
        Ok(())
    }

    pub async fn load_and_schedule_monitors(&mut self) -> Result<()> {
        let monitors = self.get_enabled_monitors().await?;
        info!("Found {} enabled monitors", monitors.len());
        
        for monitor in monitors {
            self.schedule_monitor(monitor).await?;
        }
        
        Ok(())
    }

    async fn get_enabled_monitors(&self) -> Result<Vec<Monitor>> {
        let rows = sqlx::query("SELECT * FROM monitors WHERE enabled = true")
            .fetch_all(&self.db)
            .await?;

        let mut monitors = Vec::new();
        for row in rows {
            let monitor = Monitor {
                id: row.get("id"),
                name: row.get("name"),
                endpoint: row.get("endpoint"),
                method: row.get("method"),
                headers: row.get("headers"),
                body: row.get("body"),
                expected_status: row.get("expected_status"),
                timeout: row.get("timeout"),
                interval: row.get("interval"),
                script: row.get("script"),
                enabled: row.get("enabled"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            monitors.push(monitor);
        }
        
        Ok(monitors)
    }

    async fn schedule_monitor(&mut self, monitor: Monitor) -> Result<()> {
        let db = self.db.clone();
        let client = self.http_client.clone();
        let monitor_name = monitor.name.clone();
        let interval = monitor.interval;
        
        let cron_expression = format!("0/{} * * * * *", interval);
        
        let job = Job::new_async(&cron_expression, move |_uuid, _l| {
            let db = db.clone();
            let client = client.clone();
            let monitor = monitor.clone();
            
            Box::pin(async move {
                if let Err(e) = execute_monitor_check(&db, &client, &monitor).await {
                    error!("Monitor check failed for {}: {}", monitor.name, e);
                }
            })
        })
        .map_err(|e| Error::scheduler(e.to_string()))?;
        
        self.scheduler.add(job).await
            .map_err(|e| Error::scheduler(e.to_string()))?;
        info!("Scheduled monitor: {} (interval: {}s)", monitor_name, interval);
        
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping monitor scheduler");
        self.scheduler.shutdown().await
            .map_err(|e| Error::scheduler(e.to_string()))?;
        info!("Monitor scheduler stopped");
        Ok(())
    }
}

async fn execute_monitor_check(
    db: &DatabasePool,
    client: &Client,
    monitor: &Monitor,
) -> Result<()> {
    info!("Executing monitor check: {}", monitor.name);
    
    let start_time = Instant::now();
    let mut request = client.request(
        monitor.method.parse().unwrap_or(reqwest::Method::GET),
        &monitor.endpoint,
    );
    
    if let Some(headers) = &monitor.headers {
        if let Ok(header_map) = serde_json::from_value::<std::collections::HashMap<String, String>>(headers.clone()) {
            for (key, value) in header_map {
                request = request.header(&key, &value);
            }
        }
    }
    
    if let Some(body) = &monitor.body {
        request = request.body(body.clone());
    }
    
    let result = match tokio::time::timeout(
        std::time::Duration::from_secs(monitor.timeout as u64),
        request.send(),
    ).await {
        Ok(Ok(response)) => {
            let response_time = start_time.elapsed().as_millis() as i32;
            let status_code = response.status().as_u16() as i32;
            let response_body = response.text().await.unwrap_or_default();
            
            let status = if status_code == monitor.expected_status {
                "success".to_string()
            } else {
                "failure".to_string()
            };
            
            MonitorResult {
                id: Uuid::new_v4(),
                monitor_id: monitor.id,
                status,
                response_time,
                response_code: Some(status_code),
                response_body: Some(response_body),
                error_message: None,
                checked_at: Utc::now(),
            }
        },
        Ok(Err(e)) => {
            let response_time = start_time.elapsed().as_millis() as i32;
            
            MonitorResult {
                id: Uuid::new_v4(),
                monitor_id: monitor.id,
                status: "error".to_string(),
                response_time,
                response_code: None,
                response_body: None,
                error_message: Some(e.to_string()),
                checked_at: Utc::now(),
            }
        },
        Err(_) => {
            let response_time = start_time.elapsed().as_millis() as i32;
            
            MonitorResult {
                id: Uuid::new_v4(),
                monitor_id: monitor.id,
                status: "timeout".to_string(),
                response_time,
                response_code: None,
                response_body: None,
                error_message: Some("Request timeout".to_string()),
                checked_at: Utc::now(),
            }
        }
    };
    
    save_monitor_result(db, &result).await?;
    
    if result.status != "success" {
        warn!("Monitor {} failed: {:?}", monitor.name, result.error_message);
    } else {
        info!("Monitor {} succeeded in {}ms", monitor.name, result.response_time);
    }
    
    Ok(())
}

async fn save_monitor_result(db: &DatabasePool, result: &MonitorResult) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO monitor_results (id, monitor_id, status, response_time, response_code, response_body, error_message, checked_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#
    )
    .bind(result.id)
    .bind(result.monitor_id)
    .bind(&result.status)
    .bind(result.response_time)
    .bind(result.response_code)
    .bind(&result.response_body)
    .bind(&result.error_message)
    .bind(result.checked_at)
    .execute(db)
    .await?;
    
    Ok(())
}