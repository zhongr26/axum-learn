use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{entity::todo, error::AppError};

type Todo = todo::Model;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTodo {
  pub title: String,
  pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTodo {
  pub title: Option<String>,
  pub description: Option<String>,
  pub done: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct ListQuery {
  pub done: Option<bool>,
  #[serde(default = "default_page")]
  pub page: u64,
  #[serde(default = "default_per_page")]
  pub per_page: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TodoResponse {
  pub id: i64,
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
      id: t.id as i64,
      title: t.title,
      description: t.description,
      done: t.done,
      created_at: t.created_at.into(),
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
