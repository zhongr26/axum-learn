# Axum 学习笔记

目标：参照 `architecture.svg`（Broccoli 评测系统）学习关键技术栈。

## 目标架构

```
Web frontend (React/Monaco)
      │ REST API
      ▼
Broccoli server (Axum API · auth · contests · plugin host/Extism)
      ├── PostgreSQL (schema auto-sync)
      ├── Object storage (SeaweedFS/FS/DB)
      ├── Plugins (WASM 模块，启动时加载：hooks/routes/config)
      └── Redis queue ──► Judge workers (isolate sandbox, compile/run/check, Linux only)
```

## 学习路线 / 进展

- [ ] 1. Axum 基础：路由、Handler、Extractor
- [ ] 2. 状态管理（State）、共享连接池
- [ ] 3. 错误处理（IntoResponse）、统一 API 响应
- [ ] 4. 中间件（tower / middleware）与认证（auth）
- [ ] 5. PostgreSQL 集成（sqlx / schema auto-sync）
- [ ] 6. 对象存储集成（SeaweedFS / S3 兼容）
- [ ] 7. Redis 队列与任务派发（worker 模式）
- [ ] 8. WASM 插件系统（Extism / wasmtime）
- [ ] 9. 沙箱 worker（isolate sandbox，compile/run/check）

## 关键知识点

### 1. 启动套路（main）
```rust
let state = Arc::new(AppState::new());
let app = Router::new()
    .nest("/api/todos", todos::router())   // 子路由
    .fallback(not_found)                   // 404 兜底
    .with_state(state)
    .layer(TraceLayer::new_for_http());    // 洋葱模型：后 .layer() 在外层
axum::serve(listener, app).with_graceful_shutdown(ctrl_c()).await;
```

### 2. Handler = Extractor 参数 → IntoResponse 返回
```rust
async fn get_one(
    State(state): State<Arc<AppState>>,   // 共享状态
    Path(id): Path<u64>,                  // 路径参数，自动解析，失败 400
    Query(q): Query<ListQuery>,           // 查询参数，字段全 Option
    Json(body): Json<CreateTodo>,         // body，非法自动 422
) -> Result<(StatusCode, Json<Todo>), AppError> { ... }
```
- extractor 顺序规则：最多一个 body 类型的 extractor（Json），且必须是最后一个参数。
- 自定义类型做 Path 需实现 `FromStr`；做 Json 需要 `Deserialize`。

### 3. 错误处理：自定义 enum 实现 IntoResponse
```rust
enum AppError { NotFound(String), BadRequest(String), Unauthorized, Internal(anyhow::Error) }
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self { /* 500 只记日志不泄露细节 */ };
        (status, Json(json!({"error": {"message": msg}}))).into_response()
    }
}
// From<xxx> for AppError 让业务代码直接 `?` 传播
```

### 4. 中间件
```rust
async fn require_auth(State(st): State<Arc<AppState>>, req: Request, next: Next)
    -> Result<Response, AppError> { /* 校验失败早退 */ Ok(next.run(req).await) }

// 路由组套用：
.route_layer(from_fn_with_state(state.clone(), require_auth))
```
- 中间件也是 handler：`Request` 进 `Next` 出。
- 中间件往 `request.extensions_mut().insert(x)` 塞数据，handler 用 `Extension<T>` 提取。
- `route_layer` 只作用于本路由组，`layer` 作用于整个 Router。

### 5. 状态与并发
- AppState 结构体持锁/连接池，外部 `Arc` 包裹；`RwLock` 锁内不做 await（做 IO 换 `tokio::sync::RwLock`）。
- ID 自增用 `AtomicU64`。

### 6. 测试：不起端口直接打 Router
```rust
use tower::ServiceExt;
let res = app.clone().oneshot(Request::builder().uri("/api/todos/1").body(Body::empty())?).await?;
```
- 要求路由组装抽成 `pub fn build_app(state) -> Router`，项目用 lib.rs + main.rs 结构。

## 项目进展

todo-api 项目按概念拆为 5 个递进阶段（每阶段引入概念是下一阶段依赖）：

- [~] 阶段 1 · 路由与 Handler：route/nest/fallback、Path/Query/Json、元组返回；错误先 unwrap
  ```rust
  // ⚠️ axum 0.8 路径参数语法是 /{id}，0.7 的 /:id 会运行时 panic（路由是运行时构造的）
  Router::new().route("/", get(list).post(create))
      .route("/{id}", get(get_one)).nest("/api", sub).fallback(not_found)
  // extractor 顺序：Path/Query 在前，body 类(Json)必须是最后一个参数
  async fn get_one(Path(id): Path<u64>) -> Json<Todo>
  // 返回即响应：凡 IntoResponse 均可返回；元组层层拼接
  (StatusCode::CREATED, Json(todo))
  // axum 自动错误：路径参数失败→400，body 反序列化失败→422（handler 无感知）
  // 模块组织（无 mod.rs）：src/routes.rs 作为入口 + src/routes/ 目录存子模块，同名共存
  ```
- [~] 阶段 2 · 错误处理与分层：AppError + IntoResponse（axum 版 @ControllerAdvice）+ 分层架构
  ```rust
  // 统一错误：enum + IntoResponse，From 链让 ? 传播；Internal 细节只进日志
  enum AppError { NotFound(String), Validation(String), Conflict(String), Internal(anyhow::Error) }
  impl IntoResponse for AppError { /* status + json!({"error":{...}}) */ }

  // 分层对照 Spring：routes(Controller) → service(BO) → repository(DAO) → model(Entity)；dto 与实体解耦
  // handler 固定形态：extractor 进 → 调 service → IntoResponse 出，业务逻辑不进 handler
  // 换数据库只改 repository 实现，service/routes 不动 —— 分层的意义

  // "Spring 注解"替代：无 @GetMapping 式宏（刻意设计）
  //   #[debug_handler]  编译期把 extractor 类型错误翻译成人话 (axum-macros)
  //   #[derive(IntoResponse)] 派生错误响应
  //   #[utoipa::path]   OpenAPI 文档，最接近注解体验
  ```
- [ ] 阶段 2 · 错误处理：AppError + IntoResponse + From 链让 `?` 传播；404/400/422 语义
- [ ] 阶段 3 · State 与数据形态：Arc<AppState>、with_state、RwLock 取舍、DTO 分离、分页、FromStr 做 Path
- [ ] 阶段 4 · 中间件与认证：from_fn(_with_state)、洋葱顺序、route_layer、Extension、tower-http 三件套
- [ ] 阶段 5 · 可测试性与工程化：lib.rs+main.rs 拆分、build_app()、oneshot 集成测试、优雅停机、内存 store 抽 trait（通往 sqlx/Redis 的桥）
