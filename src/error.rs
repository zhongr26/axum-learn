use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
  /// 数据不存在 -> 404
  NotFound(String),
  /// 参数校验失败 -> 400
  Validation(String),
  /// 资源冲突（如重复创建）-> 409
  Conflict(String),
  /// 未预料错误 -> 500，细节只进日志
  Internal(anyhow::Error),
}

impl IntoResponse for AppError {
  fn into_response(self) -> axum::response::Response {
    let (status, code, message) = match &self {
      AppError::NotFound(m) => (StatusCode::NOT_FOUND, "NOT_FOUND", m.clone()),
      AppError::Validation(m) => (StatusCode::BAD_REQUEST, "VALIDATION", m.clone()),
      AppError::Conflict(m) => (StatusCode::CONFLICT, "CONFLICT", m.clone()),
      AppError::Internal(e) => {
        tracing::error!("internal error: {e:?}");
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          "INTERNAL",
          "internal server error".to_string(), // 不泄露细节
        )
      }
    };
    (
      status,
      Json(json!({
        "error": {
          "code": code,
          "message": message
        }
      })),
    )
      .into_response()
  }
}

impl From<anyhow::Error> for AppError {
  fn from(e: anyhow::Error) -> Self {
    Self::Internal(e)
  }
}

impl From<std::io::Error> for AppError {
  fn from(e: std::io::Error) -> Self {
    Self::Internal(e.into())
  }
}

impl AppError {
  pub fn not_found(msg: impl Into<String>) -> Self {
    Self::NotFound(msg.into())
  }

  pub fn validation(msg: impl Into<String>) -> Self {
    Self::Validation(msg.into())
  }

  pub fn conflict(msg: impl Into<String>) -> Self {
    Self::Conflict(msg.into())
  }
}
