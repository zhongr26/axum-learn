use axum_learn::{
  build_app,
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
