use crate::store::todo::TodoStore;

#[derive(Clone)]
pub struct AppState {
  // 阶段5替换为 DatabaseConnection
  pub todos: TodoStore,
  // 阶段5替换为 env 加载
  pub config: Config,
}

#[derive(Clone)]
pub struct Config {
  pub auth_token: String,
}

impl Config {
  pub fn dev() -> Self {
    Self {
      auth_token: "secret-token".to_string(),
    }
  }
}
