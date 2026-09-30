use axum::{body::Body, extract::Request, http::header, response::Response};
use axum_learn::{
  build_app,
  state::{AppState, Config},
  store::todo::TodoStore,
};
use http_body_util::BodyExt;
use serde_json::Value;

mod handlers;

fn app() -> axum::Router {
  build_app(AppState {
    todos: TodoStore::new(),
    config: Config::dev(),
  })
}

fn req(method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> Request<Body> {
  let mut builder = Request::builder().method(method).uri(uri);

  if let Some(token) = token {
    builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
  }
  if let Some(body) = body {
    builder = builder.header(header::CONTENT_TYPE, "application/json");
    builder.body(Body::from(body.to_string())).unwrap()
  } else {
    builder.body(Body::empty()).unwrap()
  }
}

async fn body_json(res: Response) -> Value {
  let bytes = res.into_body().collect().await.unwrap().to_bytes();
  serde_json::from_slice(&bytes).unwrap()
}
