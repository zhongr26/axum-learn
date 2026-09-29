mod entity;
mod error;
mod models;
mod repository;
mod response;
mod routes;
mod service;

use axum::{Router, http::StatusCode, routing::get};

/// axum 的启动套路：
/// 1. 构造共享状态 AppState（一般用 Arc 包裹）
/// 2. 用 Router::new() 声明路由，.with_state() 注入状态
/// 3. 叠加中间件（layer 的执行顺序：后 .layer() 的先执行，即"洋葱模型"外层）
/// 4. tokio::net::TcpListener + axum::serve
#[tokio::main]
async fn main() {
  let app = Router::new()
    .route("/", get(|| async { "Hello, World" }))
    .nest("/api/todos", routes::todo::router())
    .nest("/api/health", routes::health::router())
    // 没有任何一个路由匹配到时
    .fallback(|| async { (StatusCode::NOT_FOUND, "route not found") });

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  println!("Listening on https://127.0.0.1:3000");

  axum::serve(listener, app).await.unwrap();
}
