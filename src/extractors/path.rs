use axum::{
  extract::{FromRequestParts, path::Path},
  response::Response,
};

use crate::extractors::json::rejection_response;

/// 包装 Path extractor：路径参数解析失败（如 /todos/abc）时返回统一 JSON 错误体，
/// 而不是 axum 默认的纯文本 400。
pub struct AppPath<T>(pub T);

impl<S, T> FromRequestParts<S> for AppPath<T>
where
  S: Send + Sync,
  T: serde::de::DeserializeOwned + Send,
{
  type Rejection = Response;

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    let Path(value) = Path::<T>::from_request_parts(parts, state)
      .await
      .map_err(|e| rejection_response(e.status(), e.body_text()))?;
    Ok(AppPath(value))
  }
}
