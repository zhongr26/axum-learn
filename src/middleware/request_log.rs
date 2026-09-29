use std::time::Instant;

use axum::{
  extract::Request,
  http::HeaderName,
  middleware::Next,
  response::Response,
};

static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// from_fn 中间件三件套：Request 进 -> 自定义逻辑 -> next.run(req) 出。
/// 与 extractor 的区别：它是"包住所有 handler"的外层洋葱皮，适合日志/限流/超时
pub async fn request_log(req: Request, next: Next) -> Response {
  let start = Instant::now();
  let method = req.method().clone();
  let path = req.uri().path().to_string();
  let req_id = req
    .headers()
    .get(&X_REQUEST_ID)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("-")
    .to_string();
  let res = next.run(req).await;

  tracing::info!(
    req_id, %method, path, status = res.status().as_u16(), elapsed_ms = start.elapsed().as_millis() as u64, "request"
  );
  res
}
