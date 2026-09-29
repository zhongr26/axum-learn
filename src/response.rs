use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
  pub data: T,
}

impl<T> ApiResponse<T> {
  pub fn ok(data: T) -> Self {
    Self { data }
  }
}
