use std::{
  collections::HashMap,
  sync::{
    Arc, RwLock,
    atomic::{AtomicU64, Ordering},
  },
};

use crate::{
  entity::todo::Todo,
  error::AppError,
  models::todo::{CreateTodo, UpdateTodo},
};

#[derive(Clone)]
pub struct TodoStore {
  next_id: Arc<AtomicU64>,
  store: Arc<RwLock<HashMap<u64, Todo>>>,
}

impl TodoStore {
  pub fn new() -> Self {
    // ID 从 1 开始：fetch_add 返回旧值再自增，初始值 0 会导致第一个 id 为 0
    Self {
      next_id: Arc::new(AtomicU64::new(1)),
      store: Arc::default(),
    }
  }

  pub fn next_id(&self) -> u64 {
    self.next_id.fetch_add(1, Ordering::Relaxed)
  }

  pub fn list(&self, done: Option<bool>, page: u64, per_page: u64) -> (Vec<Todo>, u64) {
    let map = self.store.read().unwrap();
    let mut all: Vec<Todo> = map
      .values()
      .filter(|t| done.map_or(true, |d| t.done == d))
      .cloned()
      .collect();
    all.sort_by_key(|t| t.id);
    let total = all.len() as u64;
    let page_items = all
      .into_iter()
      .skip((page.saturating_sub(1) * per_page) as usize)
      .take(per_page as usize)
      .collect();
    (page_items, total)
  }

  pub fn get(&self, id: u64) -> Result<Todo, AppError> {
    self
      .store
      .read()
      .unwrap()
      .get(&id)
      .cloned()
      .ok_or(AppError::todo_not_found(id))
  }

  pub fn create(&self, req: CreateTodo) -> Todo {
    let todo = Todo {
      id: self.next_id(),
      title: req.title,
      description: req.description,
      done: false,
      created_at: chrono::Utc::now(),
    };
    self.store.write().unwrap().insert(todo.id, todo.clone());
    todo
  }

  pub fn update(&self, id: u64, req: UpdateTodo) -> Result<Todo, AppError> {
    let mut map = self.store.write().unwrap();
    let todo = map
      .get_mut(&id)
      .ok_or(AppError::todo_not_found(id))?;

    if let Some(title) = req.title {
      todo.title = title;
    }
    if let Some(description) = req.description {
      todo.description = Some(description);
    }
    if let Some(done) = req.done {
      todo.done = done;
    }
    Ok(todo.clone())
  }

  pub fn delete(&self, id: u64) -> Result<(), AppError> {
    self
      .store
      .write()
      .unwrap()
      .remove(&id)
      .map(|_| ())
      .ok_or_else(|| AppError::todo_not_found(id))
  }

  pub fn has_title(&self, title: &str) -> Result<bool, AppError> {
    Ok(
      self
        .store
        .read()
        .unwrap()
        .values()
        .any(|t| t.title == title),
    )
  }
}
