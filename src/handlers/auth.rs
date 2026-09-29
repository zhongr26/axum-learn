use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::json;

use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
  pub username: String,
  pub password: String,
}

/// POST /api/v1/auth.login
pub async fn login(
  State(state): State<AppState>,
  Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
  if req.password != "password" {
    return Err(AppError::Validation {
      code: "INVALID_CREDENTIALS".into(),
      message: "wrong username or password".into(),
    });
  }
  Ok((
    StatusCode::OK,
    Json(json!({
      "token": state.config.auth_token,
      "token_type": "Bearer",
      "user": req.username
    })),
  ))
}
