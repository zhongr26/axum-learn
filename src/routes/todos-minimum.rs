use std::collections::HashMap;

use axum::{
  Json, Router,
  extract::{Path, Query},
  http::StatusCode,
  routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
pub struct Todo {
  pub id: u64,
  pub title: String,
  pub done: bool,
  pub created_at: DateTime<Utc>,
}

static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

// 这是做了一个单例模式，同时配合 Mutex 实现线程共享（阶段1临时）
fn store() -> &'static Mutex<HashMap<u64, Todo>> {
  static STORE: std::sync::OnceLock<Mutex<HashMap<u64, Todo>>> = std::sync::OnceLock::new();

  STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
  pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
  pub done: Option<bool>,
}

pub fn router() -> Router<()> {
  Router::new()
    // 一条路径可以绑定多个方法
    .route("/", get(list).post(create))
    .route("/{id}", get(get_one))
}

// GET /api/todos?done=false
async fn list(Query(q): Query<ListQuery>) -> Json<Vec<Todo>> {
  // 现阶段直接 unwrap 不考虑锁的问题
  let map = store().lock().unwrap();
  let mut todos: Vec<Todo> = map
    .values()
    .filter(|t| q.done.map_or(true, |d| t.done == d))
    .cloned()
    .collect();
  todos.sort_by_key(|t| t.id);
  Json(todos)
}

// POST /api/todos body: {"title": "..."}
async fn create(Json(input): Json<CreateTodo>) -> (StatusCode, Json<Todo>) {
  let todo = Todo {
    id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
    title: input.title,
    done: false,
    created_at: Utc::now(),
  };
  store().lock().unwrap().insert(todo.id, todo.clone());
  (StatusCode::CREATED, Json(todo))
}

// GET /api/todos/:id
async fn get_one(Path(id): Path<u64>) -> Json<Todo> {
  let map = store().lock().unwrap();
  Json(map.get(&id).cloned().unwrap())
}
