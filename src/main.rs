use std::sync::Arc;

use axum_learn::{
  blob::local::LocalBlobStore,
  build_app,
  state::{AppState, Config},
};
use sea_orm_migration::MigratorTrait;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "todo_api=debug,tower_http=info".into()),
    )
    .init();

  let db = sea_orm::Database::connect(
    &std::env::var("DATABASE_URL")
      .unwrap_or_else(|_| "postgres://todo:todo@localhost:5432/todo".into()),
  )
  .await
  .unwrap();

  // schema auto-sync：启动时跑迁移，表不存在则创建
  axum_learn::migration::Migrator::up(&db, None)
    .await
    .unwrap();

  let state = AppState {
    db,
    config: Config::dev(),
    blobs: Arc::new(LocalBlobStore {
      root: "./data/blobs".into(),
    }),
  };

  let app = build_app(state);

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  println!("Listening on https://127.0.0.1:3000");

  axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

/// 优雅停机， Ctrl+C 停止接收新链接，等在途请求完成再退出
async fn shutdown_signal() {
  tokio::signal::ctrl_c()
    .await
    .expect("failed to listen for ctrl_c");
  println!("shutting down gracefully")
}
