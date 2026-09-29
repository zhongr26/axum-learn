use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Todo {
  pub id: u64,
  pub title: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
  pub done: bool,
  pub created_at: DateTime<Utc>,
}
