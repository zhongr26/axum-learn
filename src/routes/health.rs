use axum::{Json, Router, http::StatusCode, routing::get};
use serde_json::json;

pub fn router() -> Router<()> {
  Router::new().route("/", get(health))
}

async fn health() -> (StatusCode, Json<serde_json::Value>) {
  (
    StatusCode::OK,
    Json(json!({
      "status": "ok",
      "time": &&chrono::Utc::now().to_rfc3339()
    })),
  )
}
