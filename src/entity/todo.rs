use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, models::todo::Todo};

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
  pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
  pub title: Option<String>,
  pub done: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
  pub done: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct TodoResponse {
  pub id: u64,
  pub title: String,
  pub done: bool,
  pub created_at: DateTime<Utc>,
}

impl From<Todo> for TodoResponse {
  fn from(t: Todo) -> Self {
    Self {
      id: t.id,
      title: t.title,
      done: t.done,
      created_at: t.created_at,
    }
  }
}

impl CreateTodo {
  pub fn validate(&self) -> Result<(), AppError> {
    if self.title.trim().is_empty() {
      return Err(AppError::validation("title must not be empty"));
    }
    if self.title.len() > 200 {
      return Err(AppError::validation("title too long (max 200 chars)"));
    }
    Ok(())
  }
}
