use crate::{
  entity::todo::{CreateTodo, TodoResponse, UpdateTodo},
  error::AppError,
  models::todo::Todo,
  repository::todo::{self},
};

pub fn create(req: CreateTodo) -> Result<TodoResponse, AppError> {
  req.validate()?;

  // 同标题视为重复创建
  if todo::global()
    .find_all(None)
    .iter()
    .any(|t| t.title == req.title)
  {
    return Err(AppError::conflict(format!(
      "todo '{}' already exists",
      req.title
    )));
  }

  let todo = Todo {
    id: todo::global().next_id(),
    title: req.title.trim().to_string(),
    done: false,
    created_at: chrono::Utc::now(),
  };
  todo::global().save(todo.clone());
  Ok(todo.into())
}

pub fn get(id: u64) -> Result<TodoResponse, AppError> {
  Ok(todo::global().find_by_id(id)?.into())
}

pub fn list(done: Option<bool>) -> Vec<TodoResponse> {
  todo::global()
    .find_all(done)
    .into_iter()
    .map(Into::into)
    .collect()
}

pub fn update(id: u64, req: UpdateTodo) -> Result<TodoResponse, AppError> {
  let mut todo: Todo = todo::global().find_by_id(id)?;
  if let Some(title) = req.title {
    if title.trim().is_empty() {
      return Err(AppError::validation("title must not be empty"));
    }
    todo.title = title.trim().to_string();
  }
  if let Some(done) = req.done {
    todo.done = done;
  }
  Ok(todo::global().update(todo)?.into())
}

pub fn delete(id: u64) -> Result<(), AppError> {
  todo::global().delete(id)
}
