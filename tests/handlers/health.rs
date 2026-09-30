use axum::http::StatusCode;
use tower::ServiceExt;

use crate::{app, req};

#[tokio::test]
async fn health_returns_ok() {
  let res = app()
    .oneshot(req("GET", "/api/v1/health", None, None))
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);
}
