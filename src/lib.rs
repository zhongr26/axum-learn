pub mod blob;
pub mod entity;
pub mod error;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod migration;
pub mod models;
pub mod routes;
pub mod state;
pub mod store;
pub mod utils;

use std::time::Duration;

use axum::{Json, Router, http::StatusCode, routing::get};
use serde_json::json;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};

use crate::state::AppState;

pub fn build_app(state: AppState) -> Router {
  let (router, openapi) = routes::v1::router().split_for_parts();

  // routes! 注册的路径已含 /api/v1 前缀，这里直接 merge（nest 会变成 /api/v1/api/v1）
  // SwaggerUi 是 Router<()>，必须在 with_state 之后 merge
  Router::new()
    .merge(router)
    .with_state(state)
    .route("/", get(|| async { "Todo API" }))
    .merge(utoipa_swagger_ui::SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
    .fallback(|| async {
      (
        StatusCode::NOT_FOUND,
        Json(json!({
          "code":"NOT_FOUND",
          "message": "route not found"
        })),
      )
    })
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
