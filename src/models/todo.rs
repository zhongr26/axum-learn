use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{entity::todo::Todo, error::AppError};

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
  pub title: String,
  pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
  pub title: Option<String>,
  pub description: Option<String>,
  pub done: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
  pub done: Option<bool>,
  #[serde(default = "default_page")]
  pub page: u64,
  #[serde(default = "default_per_page")]
  pub per_page: u64,
}

#[derive(Debug, Serialize)]
pub struct TodoResponse {
  pub id: u64,
  pub title: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub done: bool,
  pub created_at: DateTime<Utc>,
}

fn default_page() -> u64 {
  1
}
fn default_per_page() -> u64 {
  20
}

impl From<Todo> for TodoResponse {
  fn from(t: Todo) -> Self {
    Self {
      id: t.id,
      title: t.title,
      description: t.description,
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

impl UpdateTodo {
  pub fn validate(&self) -> Result<(), AppError> {
    if let Some(title) = &self.title {
      if title.trim().is_empty() {
        return Err(AppError::validation("title must not be empty"));
      }
      if title.len() > 200 {
        return Err(AppError::validation("title too long (max 200 chars)"));
      }
    }
    Ok(())
  }
}
