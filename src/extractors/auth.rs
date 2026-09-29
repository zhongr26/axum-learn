use axum::{
  Json,
  extract::FromRequestParts,
  http::{StatusCode, header::AUTHORIZATION, request::Parts},
  response::{IntoResponse, Response},
};
use serde_json::json;

use crate::state::AppState;

/// 提取即鉴权：handler 参数里出现 AuthUser 即要求 Bearer token，
/// 校验失败返回 401，handler 不会执行。
pub struct AuthUser {
  pub username: String,
}

// axum 0.8：trait 内直接 async fn（原生 RPITIT），不需要 #[async_trait]、
// 也不需要手写 impl Future 返回类型
impl FromRequestParts<AppState> for AuthUser {
  type Rejection = Response; // 拒绝后直接给响应

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
      Some(t) if t == state.config.auth_token => Ok(AuthUser {
        username: "alice".into(),
      }),
      _ => Err(
        (
          StatusCode::UNAUTHORIZED,
          Json(json!({
            "code": "UNAUTHORIZED",
            "message": "missing or invalid token"
          })),
        )
          .into_response(),
      ),
    }
  }
}
