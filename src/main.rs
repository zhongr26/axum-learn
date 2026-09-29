mod entity;
mod error;
mod extractors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod state;
mod store;

use std::time::Duration;

use axum::{Json, Router, http::StatusCode, routing::get};
use serde_json::json;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};

use crate::{
  state::{AppState, Config},
  store::todo::TodoStore,
};

/// axum 的启动套路：
/// 1. 构造共享状态 AppState（一般用 Arc 包裹）
/// 2. 用 Router::new() 声明路由，.with_state() 注入状态
/// 3. 叠加中间件（layer 的执行顺序：后 .layer() 的先执行，即"洋葱模型"外层）
/// 4. tokio::net::TcpListener + axum::serve
#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "todo_api=debug,tower_http=info".into()),
    )
    .init();

  let state = AppState {
    todos: TodoStore::new(),
    config: Config::dev(),
  };

  let app = Router::new()
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
    .layer(CorsLayer::permissive());

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  println!("Listening on https://127.0.0.1:3000");

  axum::serve(listener, app).await.unwrap();
}
