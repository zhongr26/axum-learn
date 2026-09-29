use axum::{Router, routing};

use crate::{handlers, state::AppState};

pub fn router() -> Router<AppState> {
  Router::new()
    .route(
      "/todos",
      routing::get(handlers::todo::list_todos).post(handlers::todo::create_todo),
    )
    .route(
      "/todos/{id}",
      routing::get(handlers::todo::get_todo)
        .patch(handlers::todo::update_todo)
        .delete(handlers::todo::delete_todo),
    )
    .route("/health", routing::get(handlers::health::health))
}
