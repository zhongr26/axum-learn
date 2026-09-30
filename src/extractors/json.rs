use axum::{
  Json,
  extract::{FromRequest, Request},
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde_json::json;

/// axum 默认的 Json 拒绝格式是纯文本，和我们的错误 JSON 不统一。
/// AppJson 包装后：拒绝 -> 统一 JSON 错误体，同时保留 axum 的状态码语义
/// （语法错/缺 Content-Type=400，数据错误如缺字段/类型错=422）。
/// 用法：AppJson(req): AppJson<CreateTodo>
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
      .map_err(|e| rejection_response(e.status(), e.body_text()))?;
    Ok(AppJson(value))
  }
}

/// 拒绝统一出口：状态码跟随 axum 原始语义，错误体统一 JSON
pub fn rejection_response(status: StatusCode, message: String) -> Response {
  (status, Json(json!({ "code": "VALIDATION_ERROR", "message": message }))).into_response()
}
