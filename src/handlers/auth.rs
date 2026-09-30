use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{error::AppError, state::AppState, utils::jwt};

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
  pub username: String,
  pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
  pub token: String,
  pub token_type: String,
  pub user: String,
}

/// POST /api/v1/auth/login
/// 演示版：固定口令校验 + 签发 JWT；落地 users 表后换 password::verify(db_hash)
#[utoipa::path(
  post,
  path = "/api/v1/auth/login",
  request_body = LoginRequest,
  responses(
    (status = 200, body = LoginResponse),
    (status = 400, description = "wrong credentials")
  ),
  tag = "auth"
)]
pub async fn login(
  State(state): State<AppState>,
  Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), AppError> {
  if req.password != "password" {
    return Err(AppError::Validation {
      code: "INVALID_CREDENTIALS".into(),
      message: "wrong username or password".into(),
    });
  }
  let token = jwt::sign(&req.username, &state.config.jwt_secret, 3600)?;
  Ok((
    StatusCode::OK,
    Json(LoginResponse {
      token,
      token_type: "Bearer".into(),
      user: req.username,
    }),
  ))
}
