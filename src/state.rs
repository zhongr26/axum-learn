use std::sync::Arc;

use crate::blob::store::BlobStore;

#[derive(Clone)]
pub struct AppState {
  pub db: sea_orm::DatabaseConnection,
  pub config: Config,
  pub blobs: Arc<dyn BlobStore>, // trait object DI
}

#[derive(Clone)]
pub struct Config {
  pub jwt_secret: String,
}

impl Config {
  pub fn from_env() -> Self {
    Self {
      jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".to_string()),
    }
  }

  pub fn dev() -> Self {
    Config::from_env()
  }
}
