use std::{
  collections::HashMap,
  sync::{
    Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
  },
};

use crate::{error::AppError, models::todo::Todo};

#[derive(Default)]
pub struct TodoRepository {
  next_id: AtomicU64,
  store: Mutex<HashMap<u64, Todo>>,
}

impl TodoRepository {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn next_id(&self) -> u64 {
    self.next_id.fetch_add(1, Ordering::Relaxed)
  }

  pub fn find_all(&self, done: Option<bool>) -> Vec<Todo> {
    let map = self.store.lock().unwrap();
    let mut v: Vec<Todo> = map
      .values()
      .filter(|t| done.map_or(true, |d| t.done == d))
      .cloned()
      .collect();
    v.sort_by_key(|t| t.id);
    return v;
  }

  pub fn find_by_id(&self, id: u64) -> Result<Todo, AppError> {
    self
      .store
      .lock()
      .unwrap()
      .get(&id)
      .cloned()
      .ok_or_else(|| AppError::not_found(format!("todo {id} not found")))
  }

  pub fn save(&self, todo: Todo) {
    self.store.lock().unwrap().insert(todo.id, todo);
  }

  pub fn update(&self, todo: Todo) -> Result<Todo, AppError> {
    let mut map = self.store.lock().unwrap();
    match map.get_mut(&todo.id) {
      Some(slot) => {
        *slot = todo.clone();
        Ok(todo)
      }
      None => Err(AppError::not_found(format!("todo {} not found", todo.id))),
    }
  }

  pub fn delete(&self, id: u64) -> Result<(), AppError> {
    self
      .store
      .lock()
      .unwrap()
      .remove(&id)
      .map(|_| ())
      .ok_or_else(|| AppError::not_found(format!("todo {id} not found")))
  }
}

/// 单例模式（阶段 3 替换为 AppState）
pub fn global() -> &'static TodoRepository {
  static REPO: OnceLock<TodoRepository> = OnceLock::new();
  REPO.get_or_init(TodoRepository::new)
}
