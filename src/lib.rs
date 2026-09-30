pub mod entity;
pub mod error;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod state;
pub mod store;

use std::time::Duration;

use axum::{Json, Router, http::StatusCode, routing::get};
use serde_json::json;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};

use crate::state::{AppState, Config};

pub fn build_app(state: AppState) -> Router {
  Router::new()
    .nest("/api/v1", routes::v1::router())
    .route("/", get(|| async { "Todo API (stage 4)" }))
    .fallback(|| async {
      (
        StatusCode::NOT_FOUND,
        Json(json!({
          "code":"NOT_FOUND",
          "message": "route not found"
        })),
      )
    })
    .with_state(state)
    .layer(axum::middleware::from_fn(
      middleware::request_log::request_log,
    ))
    .layer(TraceLayer::new_for_http())
    .layer(TimeoutLayer::with_status_code(
      StatusCode::REQUEST_TIMEOUT,
      Duration::from_secs(10),
    ))
    .layer(CorsLayer::permissive())
}

impl Config {
  pub fn from_env() -> Self {
    Self {
      auth_token: std::env::var("AUTH_TOKEN").unwrap_or_else(|_| "secret-token".to_string()),
    }
  }
}
