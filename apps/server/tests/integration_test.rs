use axum::http::{HeaderName, StatusCode};
use axum_test::TestServer;
use neuralscope_server::{api, db, events::application::EventBus, AppConfig, AppState};

const COOKIE: HeaderName = HeaderName::from_static("cookie");
const API_KEY: HeaderName = HeaderName::from_static("x-api-key");

async fn test_state() -> AppState {
    let config = AppConfig::from_env().expect("config");
    let bundle = db::connect(&config).await.expect("connect to infrastructure");
    AppState::new(config, bundle.pool, bundle.redis, EventBus::new(), None)
}

#[tokio::test]
#[ignore = "requires PostgreSQL and Redis (docker compose up -d)"]
async fn readiness_returns_ready_when_dependencies_up() {
    let state = test_state().await;
    let app = api::create_router(state);
    let server = TestServer::new(app).expect("server");

    let response = server.get("/ready").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "ready");
    assert_eq!(body["checks"]["database"]["status"], "up");
    assert_eq!(body["checks"]["redis"]["status"], "up");
}

#[tokio::test]
#[ignore = "requires PostgreSQL and Redis (docker compose up -d)"]
async fn status_endpoint_returns_uptime() {
    let state = test_state().await;
    let app = api::create_router(state);
    let server = TestServer::new(app).expect("server");

    let response = server.get("/api/v1/status").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "ready");
    assert!(body["uptime_seconds"].is_number());
    assert_eq!(body["environment"], "development");
}

#[tokio::test]
#[ignore = "requires PostgreSQL and Redis (docker compose up -d)"]
async fn auth_register_login_and_access_projects() {
    let state = test_state().await;
    let app = api::create_router(state);
    let server = TestServer::new(app).expect("server");

    let email = format!("test-{}@neuralscope.dev", uuid::Uuid::new_v4());

    let register = server
        .post("/api/v1/auth/register")
        .json(&serde_json::json!({
            "email": email,
            "password": "secure-password",
            "name": "Test User"
        }))
        .await;

    register.assert_status(StatusCode::CREATED);
    let register_body: serde_json::Value = register.json();
    assert_eq!(register_body["user"]["email"], email);

    let set_cookie = register
        .headers()
        .get("set-cookie")
        .expect("session cookie")
        .to_str()
        .expect("cookie str");
    assert!(set_cookie.contains("neuralscope.session_token="));

    let create_project = server
        .post("/api/v1/projects")
        .add_header(COOKIE, set_cookie)
        .json(&serde_json::json!({ "name": "My Project" }))
        .await;

    create_project.assert_status(StatusCode::CREATED);
    let project: serde_json::Value = create_project.json();
    assert_eq!(project["name"], "My Project");
    assert_eq!(project["role"], "owner");

    let login = server
        .post("/api/v1/auth/login")
        .json(&serde_json::json!({
            "email": email,
            "password": "secure-password"
        }))
        .await;

    login.assert_status_ok();
    let login_cookie = login
        .headers()
        .get("set-cookie")
        .expect("login cookie")
        .to_str()
        .expect("cookie str");

    let me = server
        .get("/api/v1/auth/me")
        .add_header(COOKIE, login_cookie)
        .await;

    me.assert_status_ok();
    let me_body: serde_json::Value = me.json();
    assert_eq!(me_body["email"], email);

    let projects = server
        .get("/api/v1/projects")
        .add_header(COOKIE, login_cookie)
        .await;

    projects.assert_status_ok();
    let projects_body: serde_json::Value = projects.json();
    assert_eq!(projects_body["meta"]["total"], 1);
}

#[tokio::test]
#[ignore = "requires PostgreSQL and Redis (docker compose up -d)"]
async fn protected_routes_require_authentication() {
    let state = test_state().await;
    let app = api::create_router(state);
    let server = TestServer::new(app).expect("server");

    let response = server.get("/api/v1/projects").await;
    response.assert_status_unauthorized();
}

#[tokio::test]
#[ignore = "requires PostgreSQL and Redis (docker compose up -d)"]
async fn api_key_authentication_works() {
    let state = test_state().await;
    let app = api::create_router(state);
    let server = TestServer::new(app).expect("server");

    let email = format!("apikey-{}@neuralscope.dev", uuid::Uuid::new_v4());

    let register = server
        .post("/api/v1/auth/register")
        .json(&serde_json::json!({
            "email": email,
            "password": "secure-password",
            "name": "API User"
        }))
        .await;

    register.assert_status(StatusCode::CREATED);
    let cookie = register
        .headers()
        .get("set-cookie")
        .expect("cookie")
        .to_str()
        .expect("cookie str");

    let create_key = server
        .post("/api/v1/api-keys")
        .add_header(COOKIE, cookie)
        .json(&serde_json::json!({ "name": "CI Key" }))
        .await;

    create_key.assert_status(StatusCode::CREATED);
    let key_body: serde_json::Value = create_key.json();
    let raw_key = key_body["key"].as_str().expect("raw key");
    assert!(raw_key.starts_with("ns_"));

    let projects = server
        .get("/api/v1/projects")
        .add_header(API_KEY, raw_key)
        .await;

    projects.assert_status_ok();
}

#[tokio::test]
#[ignore = "requires PostgreSQL and Redis (docker compose up -d)"]
async fn log_ingestion_and_search() {
    let state = test_state().await;
    let app = api::create_router(state);
    let server = TestServer::new(app).expect("server");

    let email = format!("logs-{}@neuralscope.dev", uuid::Uuid::new_v4());

    let register = server
        .post("/api/v1/auth/register")
        .json(&serde_json::json!({
            "email": email,
            "password": "secure-password",
            "name": "Log Tester"
        }))
        .await;

    register.assert_status(StatusCode::CREATED);
    let cookie = register
        .headers()
        .get("set-cookie")
        .expect("cookie")
        .to_str()
        .expect("cookie str");

    let project = server
        .post("/api/v1/projects")
        .add_header(COOKIE, cookie)
        .json(&serde_json::json!({ "name": "Logs Project" }))
        .await;

    project.assert_status(StatusCode::CREATED);
    let project_body: serde_json::Value = project.json();
    let project_id = project_body["id"].as_str().expect("project id");

    let ingest = server
        .post(&format!("/api/v1/projects/{project_id}/logs"))
        .add_header(COOKIE, cookie)
        .json(&serde_json::json!({
            "level": "error",
            "message": "Database connection timeout",
            "service": "api-gateway"
        }))
        .await;

    ingest.assert_status(StatusCode::CREATED);
    let log: serde_json::Value = ingest.json();
    assert_eq!(log["message"], "Database connection timeout");
    assert_eq!(log["level"], "error");

    let search = server
        .get(&format!("/api/v1/projects/{project_id}/logs?search=database"))
        .add_header(COOKIE, cookie)
        .await;

    search.assert_status_ok();
    let search_body: serde_json::Value = search.json();
    assert_eq!(search_body["meta"]["total"], 1);
}

#[tokio::test]
#[ignore = "requires PostgreSQL and Redis (docker compose up -d)"]
async fn metrics_and_traces_ingestion() {
    let state = test_state().await;
    let app = api::create_router(state);
    let server = TestServer::new(app).expect("server");

    let email = format!("telemetry-{}@neuralscope.dev", uuid::Uuid::new_v4());

    let register = server
        .post("/api/v1/auth/register")
        .json(&serde_json::json!({
            "email": email,
            "password": "secure-password",
            "name": "Telemetry Tester"
        }))
        .await;

    register.assert_status(StatusCode::CREATED);
    let cookie = register
        .headers()
        .get("set-cookie")
        .expect("cookie")
        .to_str()
        .expect("cookie str");

    let project = server
        .post("/api/v1/projects")
        .add_header(COOKIE, cookie)
        .json(&serde_json::json!({ "name": "Telemetry Project" }))
        .await;

    project.assert_status(StatusCode::CREATED);
    let project_body = project.json::<serde_json::Value>();
    let project_id = project_body["id"].as_str().expect("project id");

    let metric = server
        .post(&format!("/api/v1/projects/{project_id}/metrics"))
        .add_header(COOKIE, cookie)
        .json(&serde_json::json!({
            "name": "cpu.usage",
            "value": 72.5,
            "unit": "percent"
        }))
        .await;

    metric.assert_status(StatusCode::CREATED);

    let metrics = server
        .get(&format!("/api/v1/projects/{project_id}/metrics?name=cpu.usage"))
        .add_header(COOKIE, cookie)
        .await;

    metrics.assert_status_ok();
    assert_eq!(metrics.json::<serde_json::Value>()["meta"]["total"], 1);

    let trace = server
        .post(&format!("/api/v1/projects/{project_id}/traces"))
        .add_header(COOKIE, cookie)
        .json(&serde_json::json!({
            "traceId": "abc123trace",
            "spans": [
                {
                    "spanId": "root",
                    "service": "api-gateway",
                    "operation": "GET /users",
                    "durationMs": 120.0,
                    "status": "ok"
                },
                {
                    "spanId": "child",
                    "parentSpanId": "root",
                    "service": "users-service",
                    "operation": "db.query",
                    "durationMs": 45.0,
                    "status": "ok"
                }
            ]
        }))
        .await;

    trace.assert_status(StatusCode::CREATED);
    let trace_body = trace.json::<serde_json::Value>();
    assert_eq!(trace_body["spanCount"], 2);
    assert_eq!(trace_body["rootService"], "api-gateway");

    let traces = server
        .get(&format!("/api/v1/projects/{project_id}/traces"))
        .add_header(COOKIE, cookie)
        .await;

    traces.assert_status_ok();
    assert_eq!(traces.json::<serde_json::Value>()["meta"]["total"], 1);
}

#[tokio::test]
#[ignore = "requires PostgreSQL, Redis, and configured AI provider"]
async fn chat_completion_requires_ai_provider() {
    let state = test_state().await;
    let app = api::create_router(state);
    let server = TestServer::new(app).expect("server");

    let email = format!("chat-{}@neuralscope.dev", uuid::Uuid::new_v4());

    let register = server
        .post("/api/v1/auth/register")
        .json(&serde_json::json!({
            "email": email,
            "password": "secure-password",
            "name": "Chat Tester"
        }))
        .await;

    register.assert_status(StatusCode::CREATED);
    let cookie = register
        .headers()
        .get("set-cookie")
        .expect("cookie")
        .to_str()
        .expect("cookie str");

    let project = server
        .post("/api/v1/projects")
        .add_header(COOKIE, cookie)
        .json(&serde_json::json!({ "name": "Chat Project" }))
        .await;

    project.assert_status(StatusCode::CREATED);
    let project_body = project.json::<serde_json::Value>();
    let project_id = project_body["id"].as_str().expect("project id");

    let chat = server
        .post(&format!("/api/v1/projects/{project_id}/chat/completions"))
        .add_header(COOKIE, cookie)
        .json(&serde_json::json!({
            "message": "What errors happened recently?"
        }))
        .await;

    chat.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
}
