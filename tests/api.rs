use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt; // oneshot

use axum_learn::{
    build_app,
    state::{AppState, Config},
};

/// 连接 DB 并构建 app；DB 不可达时返回 None，测试打 skip。
/// 标题用 uuid 保证唯一，测试之间互不影响。
async fn app() -> Option<axum::Router> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://todo:todo@localhost:5432/todo".into());
    let db = match sea_orm::Database::connect(&url).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("skip: database unreachable ({e})");
            return None;
        }
    };
    use sea_orm_migration::MigratorTrait;
    axum_learn::migration::Migrator::up(&db, None).await.unwrap();

    Some(build_app(AppState {
        db,
        config: Config::dev(),
        blobs: std::sync::Arc::new(axum_learn::blob::local::LocalBlobStore {
            root: std::env::temp_dir().join("axum-learn-test-blobs"),
        }),
    }))
}

fn req(method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    if let Some(b) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        builder.body(Body::from(b.to_string())).unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    }
}

async fn body_json(res: axum::response::Response) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn login(app: &axum::Router) -> String {
    let res = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/auth/login",
            None,
            Some(json!({"username": "alice", "password": "password"})),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    body_json(res).await["token"].as_str().unwrap().to_string()
}

fn unique_title() -> String {
    format!("test-{}", uuid::Uuid::new_v4())
}

#[tokio::test]
async fn health_returns_ok() {
    let Some(app) = app().await else { return };
    let res = app.oneshot(req("GET", "/api/v1/health", None, None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    let Some(app) = app().await else { return };
    let res = app
        .oneshot(req(
            "POST",
            "/api/v1/auth/login",
            None,
            Some(json!({"username": "alice", "password": "wrong"})),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(res).await["code"], "INVALID_CREDENTIALS");
}

#[tokio::test]
async fn create_requires_valid_jwt() {
    let Some(app) = app().await else { return };
    let res = app
        .oneshot(req(
            "POST",
            "/api/v1/todos",
            Some("not-a-jwt"),
            Some(json!({"title": "x"})),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_and_get_todo() {
    let Some(app) = app().await else { return };
    let token = login(&app).await;
    let title = unique_title();

    let res = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/todos",
            Some(&token),
            Some(json!({"title": title})),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let created = body_json(res).await;
    assert!(created["id"].as_i64().unwrap() > 0); // DB 自增主键

    let id = created["id"].as_i64().unwrap();
    let res = app
        .clone()
        .oneshot(req("GET", &format!("/api/v1/todos/{id}"), None, None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(body_json(res).await["title"], title.as_str());
}

#[tokio::test]
async fn get_missing_todo_returns_404_with_json_error() {
    let Some(app) = app().await else { return };
    let res = app.oneshot(req("GET", "/api/v1/todos/999999", None, None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    assert_eq!(body_json(res).await["code"], "TODO_NOT_FOUND");
}

#[tokio::test]
async fn invalid_json_returns_unified_error_with_422() {
    let Some(app) = app().await else { return };
    let token = login(&app).await;
    let res = app
        .oneshot(req(
            "POST",
            "/api/v1/todos",
            Some(&token),
            Some(json!({})), // 缺 title -> 数据错误 -> 422（回归：rejection 保留原始状态码）
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body_json(res).await["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn duplicate_title_conflicts() {
    let Some(app) = app().await else { return };
    let token = login(&app).await;
    let title = unique_title();
    let body = json!({"title": title});

    app.clone()
        .oneshot(req("POST", "/api/v1/todos", Some(&token), Some(body.clone())))
        .await
        .unwrap();
    let res = app
        .oneshot(req("POST", "/api/v1/todos", Some(&token), Some(body)))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn delete_is_soft_and_then_404() {
    let Some(app) = app().await else { return };
    let token = login(&app).await;
    let title = unique_title();

    let res = app
        .clone()
        .oneshot(req(
            "POST",
            "/api/v1/todos",
            Some(&token),
            Some(json!({"title": title})),
        ))
        .await
        .unwrap();
    let id = body_json(res).await["id"].as_i64().unwrap();

    let res = app
        .clone()
        .oneshot(req("DELETE", &format!("/api/v1/todos/{id}"), Some(&token), None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    let res = app
        .oneshot(req("GET", &format!("/api/v1/todos/{id}"), None, None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND); // 软删除后查询过滤
}

#[tokio::test]
async fn openapi_doc_is_served() {
    let Some(app) = app().await else { return };
    let res = app.oneshot(req("GET", "/api-docs/openapi.json", None, None)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let doc = body_json(res).await;
    assert!(doc["paths"]["/api/v1/todos"].is_object()); // routes! 生成的文档条目
}
