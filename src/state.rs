use crate::store::todo::TodoStore;

#[derive(Clone)]
pub struct AppState {
  // 阶段5替换为 DatabaseConnection
  pub todos: TodoStore,
}
