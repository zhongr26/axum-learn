use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::{handlers, state::AppState};

/// OpenAPI 文档与路由同源：routes! 宏从 #[utoipa::path] 的 path 属性
/// 同时注册路由和文档条目。注意 path 必须写完整路径 /api/v1/...，
/// lib.rs 里直接 merge（不能 nest，否则前缀重复）。
#[derive(OpenApi)]
#[openapi(
  info(title = "Todo API", version = "1.0.0"),
  components(schemas(
    crate::models::todo::CreateTodo,
    crate::models::todo::UpdateTodo,
    crate::models::todo::ListQuery,
    crate::models::todo::TodoResponse,
    crate::handlers::auth::LoginRequest,
    crate::handlers::auth::LoginResponse,
  )),
  tags((name = "todos"), (name = "auth"))
)]
pub struct ApiDoc;

pub fn router() -> OpenApiRouter<AppState> {
  OpenApiRouter::with_openapi(ApiDoc::openapi())
    .routes(utoipa_axum::routes!(handlers::health::health))
    .routes(utoipa_axum::routes!(handlers::auth::login))
    .routes(utoipa_axum::routes!(handlers::todo::list_todos))
    .routes(utoipa_axum::routes!(handlers::todo::create_todo))
    .routes(utoipa_axum::routes!(handlers::todo::get_todo))
    .routes(utoipa_axum::routes!(handlers::todo::update_todo))
    .routes(utoipa_axum::routes!(handlers::todo::delete_todo))
}
