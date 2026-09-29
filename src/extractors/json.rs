use axum::{
  Json,
  extract::{FromRequest, Request},
  response::{IntoResponse, Response},
};

use crate::error::AppError;

/// 包装 axum 的 Json extractor：反序列化失败时返回统一的 JSON 错误体，
/// 而不是 axum 默认的纯文本（"Failed to deserialize the JSON body..."）。
/// 用法：`AppJson(req): AppJson<CreateTodo>`
pub struct AppJson<T>(pub T);

impl<S, T> FromRequest<S> for AppJson<T>
where
  S: Send + Sync,
  T: serde::de::DeserializeOwned,
{
  type Rejection = Response;

  async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let Json(value) = Json::<T>::from_request(req, state)
      .await
      .map_err(|e| AppError::validation(e.body_text()).into_response())?;
    Ok(AppJson(value))
  }
}
