use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::routing;
use axum::{Json, Router};

use crate::entity::todo::{CreateTodo, ListQuery, TodoResponse, UpdateTodo};
use crate::error::AppError;
use crate::response::ApiResponse;
use crate::service::todo;

pub fn router() -> Router<()> {
  Router::new()
    .route("/", routing::get(list).post(create))
    .route("/{id}", routing::get(get_one).patch(update).delete(delete))
}

async fn list(
  Query(q): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<TodoResponse>>>, AppError> {
  Ok(Json(ApiResponse::ok(todo::list(q.done))))
}

async fn create(
  Json(req): Json<CreateTodo>,
) -> Result<(StatusCode, Json<ApiResponse<TodoResponse>>), AppError> {
  let todo = todo::create(req)?;
  Ok((StatusCode::CREATED, Json(ApiResponse::ok(todo))))
}

async fn get_one(Path(id): Path<u64>) -> Result<Json<ApiResponse<TodoResponse>>, AppError> {
  Ok(Json(ApiResponse::ok(todo::get(id)?)))
}

async fn update(
  Path(id): Path<u64>,
  Json(req): Json<UpdateTodo>,
) -> Result<Json<ApiResponse<TodoResponse>>, AppError> {
  Ok(Json(ApiResponse::ok(todo::update(id, req)?)))
}

async fn delete(Path(id): Path<u64>) -> Result<StatusCode, AppError> {
  todo::delete(id)?;
  Ok(StatusCode::NO_CONTENT)
}
