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
#[debug_handler]
pub async fn list_todos(
  State(state): State<AppState>,
  Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, AppError> {
  let per_page = q.per_page.clamp(1, 100);
  let (todos, total) = state.todos.list(q.done, q.page, per_page);
  let data: Vec<TodoResponse> = todos.into_iter().map(Into::into).collect();

  Ok((
    [(
      X_TOTAL_COUNT.clone(),
      HeaderValue::from_str(&total.to_string()).expect("total is a number, always valid."),
    )],
    Json(data),
  ))
}

/// GET /api/v1/todos/{id}
#[debug_handler]
pub async fn get_todo(
  State(state): State<AppState>,
  AppPath(id): AppPath<u64>,
) -> Result<Json<TodoResponse>, AppError> {
  Ok(Json(state.todos.get(id)?.into()))
}

/// POST /api/v1/todos -> 201 Created
#[debug_handler]
pub async fn create_todo(
  user: AuthUser,
  State(state): State<AppState>,
  AppJson(req): AppJson<CreateTodo>,
) -> Result<(StatusCode, Json<TodoResponse>), AppError> {
  tracing::debug!(user=%user.username,"create todo");
  req.validate()?;
  if state.todos.has_title(&req.title)? == true {
    return Err(AppError::conflict(format!(
      "title {} already exists",
      req.title
    )));
  }
  Ok((StatusCode::CREATED, Json(state.todos.create(req).into())))
}

/// PATCH /api/v1/todos/{id}
#[debug_handler]
pub async fn update_todo(
  user: AuthUser,
  State(state): State<AppState>,
  AppPath(id): AppPath<u64>,
  AppJson(req): AppJson<UpdateTodo>,
) -> Result<Json<TodoResponse>, AppError> {
  tracing::debug!(user=%user.username,"create todo");
  req.validate()?;
  if let Some(title) = &req.title {
    if state.todos.has_title(&title)? == true {
      return Err(AppError::conflict(format!(
        "title {} already exists",
        title
      )));
    }
  }
  Ok(Json(state.todos.update(id, req)?.into()))
}

/// DELETE /api/v1/todos/{id} -> 204 No Content 无body
#[debug_handler]
pub async fn delete_todo(
  user: AuthUser,
  State(state): State<AppState>,
  AppPath(id): AppPath<u64>,
) -> Result<StatusCode, AppError> {
  tracing::debug!(user=%user.username,"create todo");
  state.todos.delete(id)?;
  Ok(StatusCode::NO_CONTENT)
}
