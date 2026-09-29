use axum::{
  Json,
  extract::{FromRequest, Request},
  response::{IntoResponse, Response},
};

use crate::error::AppError;

/// axum 默认的 Json 拒绝格式是纯文本，和我们的错误 JSON 不统一。
/// AppJson 包装后：反序列化失败 -> AppError::Validation -> 统一错误体。
/// 用法：Json(req): AppJson<CreateTodoRequest>
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
