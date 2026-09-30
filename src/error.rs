use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("{message}")]
  NotFound { code: String, message: String },
  #[error("{message}")]
  Validation { code: String, message: String },
  #[error("{message}")]
  Conflict { code: String, message: String },
  #[error(transparent)]
  Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
  fn into_response(self) -> axum::response::Response {
    let (status, code, message) = match &self {
      AppError::NotFound { code, message } => (StatusCode::NOT_FOUND, code, message),
      AppError::Validation { code, message } => (StatusCode::BAD_REQUEST, code, message),
      AppError::Conflict { code, message } => (StatusCode::CONFLICT, code, message),
      AppError::Internal(e) => {
        tracing::error!("internal error: {e:?}");
        return (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(json!({"code": "INTERNAL_ERROR", "message": "internal server error"})),
        )
          .into_response();
      }
    };
    (
      status,
      Json(json!({
          "code": code,
          "message": message
      })),
    )
      .into_response()
  }
}

impl AppError {
  /// 资源不存在：code 带资源类型，message 带具体 id
  pub fn todo_not_found(id: u64) -> Self {
    Self::NotFound {
      code: "TODO_NOT_FOUND".into(),
      message: format!("todo {id} not found"),
    }
  }

  pub fn validation(msg: impl Into<String>) -> Self {
    Self::Validation {
      code: "VALIDATION_ERROR".into(),
      message: msg.into(),
    }
  }

  pub fn conflict(msg: impl Into<String>) -> Self {
    Self::Conflict {
      code: "CONFLICT_ERROR".into(),
      message: msg.into(),
    }
  }
}

// 数据库错误归入 Internal（500，细节只进日志）——让 store 层的 ? 直接工作
impl From<sea_orm::DbErr> for AppError {
  fn from(e: sea_orm::DbErr) -> Self {
    Self::Internal(e.into())
  }
}

