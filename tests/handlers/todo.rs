use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use crate::{app, body_json, req};

#[tokio::test]
async fn create_and_get_todo() {
  let app = app();

  // 创建 -> 201，返回带 id
  let res = app
    .clone()
    .oneshot(req(
      "POST",
      "/api/v1/todos",
      Some("secret-token"),
      Some(json!({"title": "learn testing"})),
    ))
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::CREATED);
  let created = body_json(res).await;
  let id = created["id"].as_u64().unwrap();
  assert_eq!(id, 1); // ID 从 1 开始（踩坑修复的回归测试）

  // 读取 -> 200，字段一致；X-Total-Count 出现在列表
  let res = app
    .clone()
    .oneshot(req("GET", &format!("/api/v1/todos/{id}"), None, None))
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::OK);

  let res = app
    .oneshot(req("GET", "/api/v1/todos", None, None))
    .await
    .unwrap();
  assert_eq!(res.headers()["x-total-count"], "1");
}

#[tokio::test]
async fn get_missing_todo_returns_404_with_json_error() {
  let res = app()
    .oneshot(req("GET", "/api/v1/todos/999", None, None))
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);
  let body = body_json(res).await;
  assert_eq!(body["code"], "TODO_NOT_FOUND"); // 错误体统一为 JSON
}

#[tokio::test]
async fn invalid_json_returns_unified_error() {
  let res = app()
    .oneshot(req(
      "POST",
      "/api/v1/todos",
      Some("secret-token"),
      Some(json!({})),
    ))
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
  let body = body_json(res).await;
  assert_eq!(body["code"], "VALIDATION_ERROR"); // AppJson 接管纯文本拒绝的回归测试
}

#[tokio::test]
async fn duplicate_title_conflicts() {
  let app = app();
  let body = json!({"title": "dup"});
  app
    .clone()
    .oneshot(req(
      "POST",
      "/api/v1/todos",
      Some("secret-token"),
      Some(body.clone()),
    ))
    .await
    .unwrap();
  let res = app
    .oneshot(req(
      "POST",
      "/api/v1/todos",
      Some("secret-token"),
      Some(body),
    ))
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn delete_then_get_404() {
  let app = app();
  app
    .clone()
    .oneshot(req(
      "POST",
      "/api/v1/todos",
      Some("secret-token"),
      Some(json!({"title": "t"})),
    ))
    .await
    .unwrap();
  let res = app
    .clone()
    .oneshot(req("DELETE", "/api/v1/todos/1", Some("secret-token"), None))
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NO_CONTENT); // 204 且无 body
  let res = app
    .oneshot(req("GET", "/api/v1/todos/1", None, None))
    .await
    .unwrap();
  assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
