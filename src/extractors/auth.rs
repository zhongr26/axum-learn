use axum::{
  Json,
  extract::FromRequestParts,
  http::{StatusCode, header::AUTHORIZATION, request::Parts},
  response::{IntoResponse, Response},
};
use serde_json::json;

use crate::state::AppState;

/// 提取即鉴权：handler 参数里出现 AuthUser 即要求有效 Bearer JWT，
/// 校验失败返回 401，handler 不会执行。
pub struct AuthUser {
  pub username: String,
}

impl FromRequestParts<AppState> for AuthUser {
  type Rejection = Response;

  async fn from_request_parts(
    parts: &mut Parts,
    state: &AppState,
  ) -> Result<Self, Self::Rejection> {
    let token = parts
      .headers
      .get(AUTHORIZATION)
      .and_then(|v| v.to_str().ok())
      .and_then(|v| v.strip_prefix("Bearer "));

    match token {
      // JWT 过期/签名错误由 exp 自动校验兜住
      Some(t) => match crate::utils::jwt::verify(t, &state.config.jwt_secret) {
        Some(claims) => Ok(AuthUser { username: claims.sub }),
        None => Err(unauthorized("invalid or expired token")),
      },
      None => Err(unauthorized("missing or invalid token")),
    }
  }
}

fn unauthorized(message: &str) -> Response {
  (
    StatusCode::UNAUTHORIZED,
    Json(json!({ "code": "UNAUTHORIZED", "message": message })),
  )
    .into_response()
}
