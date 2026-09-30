use axum::{
  Json,
  extract::{Query, State},
  http::{HeaderName, HeaderValue, StatusCode},
  response::IntoResponse,
};
use axum_macros::debug_handler;

use crate::{
  error::AppError,
  extractors::{auth::AuthUser, json::AppJson, path::AppPath},
  models::todo::{CreateTodo, ListQuery, TodoResponse, UpdateTodo},
  state::AppState,
};

static X_TOTAL_COUNT: HeaderName = HeaderName::from_static("x-total-count");

/// GET /api/v1/todos?done=false&page=1&per_page=20
#[utoipa::path(
  get,
  path = "/api/v1/todos",
  params(ListQuery),
  responses((status = 200, body = [TodoResponse])),
  tag = "todos"
)]
#[debug_handler]
pub async fn list_todos(
  State(state): State<AppState>,
  Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, AppError> {
  let per_page = q.per_page.clamp(1, 100);
  let (todos, total) = crate::store::todo::list(&state.db, q.done, q.page, per_page).await?;
  let data: Vec<TodoResponse> = todos.into_iter().map(Into::into).collect();

  Ok((
    [(
      X_TOTAL_COUNT.clone(),
      HeaderValue::from_str(&total.to_string()).expect("total is a number"),
    )],
    Json(data),
  ))
}

/// GET /api/v1/todos/{id}
#[utoipa::path(
  get,
  path = "/api/v1/todos/{id}",
  params(("id" = i32, Path, description = "Todo id")),
  responses(
    (status = 200, body = TodoResponse),
    (status = 404, description = "todo not found")
  ),
  tag = "todos"
)]
#[debug_handler]
pub async fn get_todo(
  State(state): State<AppState>,
  AppPath(id): AppPath<i32>,
) -> Result<Json<TodoResponse>, AppError> {
  Ok(Json(crate::store::todo::get(&state.db, id).await?.into()))
}

/// POST /api/v1/todos -> 201 Created
#[utoipa::path(
  post,
  path = "/api/v1/todos",
  request_body = CreateTodo,
  responses(
    (status = 201, body = TodoResponse),
    (status = 400, description = "validation failed"),
    (status = 401, description = "unauthorized"),
    (status = 409, description = "duplicate title")
  ),
  tag = "todos"
)]
#[debug_handler]
pub async fn create_todo(
  user: AuthUser,
  State(state): State<AppState>,
  AppJson(req): AppJson<CreateTodo>,
) -> Result<(StatusCode, Json<TodoResponse>), AppError> {
  tracing::debug!(user = %user.username, "create todo");
  req.validate()?;
  if crate::store::todo::has_title(&state.db, &req.title).await? {
    return Err(AppError::conflict(format!(
      "title {} already exists",
      req.title
    )));
  }
  let todo = crate::store::todo::create(&state.db, req).await?;
  Ok((StatusCode::CREATED, Json(todo.into())))
}

/// PATCH /api/v1/todos/{id}
#[utoipa::path(
  patch,
  path = "/api/v1/todos/{id}",
  request_body = UpdateTodo,
  params(("id" = i32, Path)),
  responses(
    (status = 200, body = TodoResponse),
    (status = 401, description = "unauthorized"),
    (status = 404, description = "todo not found")
  ),
  tag = "todos"
)]
#[debug_handler]
pub async fn update_todo(
  user: AuthUser,
  State(state): State<AppState>,
  AppPath(id): AppPath<i32>,
  AppJson(req): AppJson<UpdateTodo>,
) -> Result<Json<TodoResponse>, AppError> {
  tracing::debug!(user = %user.username, "update todo");
  req.validate()?;
  if let Some(title) = &req.title {
    if crate::store::todo::has_title(&state.db, title).await? {
      return Err(AppError::conflict(format!("title {title} already exists")));
    }
  }
  Ok(Json(
    crate::store::todo::update(&state.db, id, req).await?.into(),
  ))
}

/// DELETE /api/v1/todos/{id} -> 204 No Content（软删除）
#[utoipa::path(
  delete,
  path = "/api/v1/todos/{id}",
  params(("id" = i32, Path)),
  responses(
    (status = 204),
    (status = 401, description = "unauthorized"),
    (status = 404, description = "todo not found")
  ),
  tag = "todos"
)]
#[debug_handler]
pub async fn delete_todo(
  user: AuthUser,
  State(state): State<AppState>,
  AppPath(id): AppPath<i32>,
) -> Result<StatusCode, AppError> {
  tracing::debug!(user = %user.username, "delete todo");
  crate::store::todo::delete(&state.db, id).await?;
  Ok(StatusCode::NO_CONTENT)
}
