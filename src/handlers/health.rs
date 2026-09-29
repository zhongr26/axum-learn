use axum::{Json, http::StatusCode};
use serde_json::json;

pub async fn health() -> (StatusCode, Json<serde_json::Value>) {
  (
    StatusCode::OK,
    Json(json!({
      "status": "ok",
      "time": &&chrono::Utc::now().to_rfc3339()
    })),
  )
}
