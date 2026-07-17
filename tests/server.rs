#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::net::TcpListener;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

fn bin_path() -> String {
    std::env::var("CARGO_BIN_EXE_review-engine").unwrap_or_else(|_| "target/debug/review-engine".to_string())
}

fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to find free port");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

struct ServerGuard {
    child: Child,
    _temp_dir: tempfile::TempDir,
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn spawn_server(port: u16) -> ServerGuard {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let child = Command::new(bin_path())
        .arg("serve")
        .arg("--bind")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .env("HOME", temp_dir.path())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn review-engine serve");
    ServerGuard {
        child,
        _temp_dir: temp_dir,
    }
}

async fn wait_for_server(port: u16) {
    let client = reqwest::Client::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match client
            .get(format!("http://127.0.0.1:{}/health", port))
            .timeout(Duration::from_millis(200))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => break,
            _ if Instant::now() > deadline => panic!("server did not start within 10 seconds"),
            _ => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
}

#[tokio::test]
async fn health_endpoint() {
    let port = find_free_port();
    let _guard = spawn_server(port);
    wait_for_server(port).await;

    let resp = reqwest::get(format!("http://127.0.0.1:{}/health", port))
        .await
        .expect("failed to call /health");
    assert!(resp.status().is_success(), "/health returned {}", resp.status());
    let body: serde_json::Value = resp.json().await.expect("/health body is not JSON");
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn health_ready_no_llm() {
    let port = find_free_port();
    let _guard = spawn_server(port);
    wait_for_server(port).await;

    let resp = reqwest::get(format!("http://127.0.0.1:{}/health/ready", port))
        .await
        .expect("failed to call /health/ready");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::SERVICE_UNAVAILABLE,
        "expected 503 from /health/ready without LLM config"
    );
    let body: serde_json::Value = resp.json().await.expect("/health/ready body is not JSON");
    assert_eq!(body["status"], "not ready");
}

#[tokio::test]
async fn metrics_endpoint() {
    let port = find_free_port();
    let _guard = spawn_server(port);
    wait_for_server(port).await;

    let resp = reqwest::get(format!("http://127.0.0.1:{}/metrics", port))
        .await
        .expect("failed to call /metrics");
    assert!(resp.status().is_success(), "/metrics returned {}", resp.status());
    let body = resp.text().await.expect("/metrics body is not text");
    assert!(
        body.contains("review_engine") || body.contains("process_"),
        "metrics did not contain expected prefix: {}",
        body
    );
}

// ─── LLM Provider CRUD ────────────────────────────────────────────

/// The frontend sends `apiBaseUrl`/`defaultModel`; the backend must map them
/// onto `api_base`/`model` via serde aliases. The primary camelCase names
/// (`apiBase`/`model`) must keep working as well.
#[test]
fn provider_requests_accept_frontend_field_aliases() {
    use review_engine::server::api::llm::{AddProviderRequest, UpdateProviderRequest};

    let add: AddProviderRequest = serde_json::from_value(serde_json::json!({
        "provider": "openai",
        "apiKey": "sk-test",
        "apiBaseUrl": "https://llm.example.test/v1",
        "defaultModel": "gpt-4o-test",
        "maxTokens": 8192,
        "temperature": 0.3,
    }))
    .expect("AddProviderRequest should accept frontend field names");
    assert_eq!(add.provider, "openai");
    assert_eq!(add.api_key, "sk-test");
    assert_eq!(add.api_base, "https://llm.example.test/v1");
    assert_eq!(add.model, "gpt-4o-test");
    assert_eq!(add.max_tokens, 8192);
    assert!((add.temperature - 0.3).abs() < f32::EPSILON);

    let add_primary: AddProviderRequest = serde_json::from_value(serde_json::json!({
        "provider": "openai",
        "apiKey": "sk-test",
        "apiBase": "https://primary.example.test/v1",
        "model": "gpt-4o-primary",
    }))
    .expect("AddProviderRequest should keep its primary camelCase names");
    assert_eq!(add_primary.api_base, "https://primary.example.test/v1");
    assert_eq!(add_primary.model, "gpt-4o-primary");

    let update: UpdateProviderRequest = serde_json::from_value(serde_json::json!({
        "apiBaseUrl": "https://update.example.test/v1",
        "defaultModel": "gpt-4o-update",
    }))
    .expect("UpdateProviderRequest should accept frontend field names");
    assert_eq!(update.api_base, "https://update.example.test/v1");
    assert_eq!(update.model, "gpt-4o-update");
}

#[tokio::test]
async fn add_provider_accepts_frontend_field_names() {
    let port = find_free_port();
    let _guard = spawn_server(port);
    wait_for_server(port).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{}/api/v1/llm/providers", port))
        .json(&serde_json::json!({
            "provider": "openai",
            "apiKey": "sk-test",
            "apiBaseUrl": "https://llm.example.test/v1",
            "defaultModel": "gpt-4o-test",
        }))
        .send()
        .await
        .expect("failed to POST /api/v1/llm/providers");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::CREATED,
        "POST /api/v1/llm/providers returned {}",
        resp.status()
    );
    let body: serde_json::Value = resp.json().await.expect("POST provider body is not JSON");
    // `defaultModel` must land in `model` — without the alias this would be "".
    assert_eq!(body["model"], "gpt-4o-test");
    assert_eq!(body["configured"], true);

    // The provider must be listed afterwards and marked as configured.
    let resp = reqwest::get(format!("http://127.0.0.1:{}/api/v1/llm/providers", port))
        .await
        .expect("failed to GET /api/v1/llm/providers");
    assert!(resp.status().is_success(), "GET providers returned {}", resp.status());
    let body: serde_json::Value = resp.json().await.expect("GET providers body is not JSON");
    let items = body["items"].as_array().expect("items is not an array");
    let added = items
        .iter()
        .find(|item| item["name"] == "openai")
        .expect("added provider missing from GET /providers");
    assert_eq!(added["configured"], true);
}

#[tokio::test]
async fn add_provider_missing_provider_field_returns_400_json() {
    let port = find_free_port();
    let _guard = spawn_server(port);
    wait_for_server(port).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{}/api/v1/llm/providers", port))
        .json(&serde_json::json!({
            "apiKey": "sk-test",
        }))
        .send()
        .await
        .expect("failed to POST /api/v1/llm/providers");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "expected 400 for a body missing `provider`, got {}",
        resp.status()
    );
    let body: serde_json::Value = resp.json().await.expect("400 response body should be JSON");
    let error = body["error"].as_str().expect("400 body must contain an `error` string");
    assert!(
        error.contains("provider"),
        "error message should mention the missing field: {}",
        error
    );
}

#[tokio::test]
async fn update_provider_accepts_frontend_field_names() {
    let port = find_free_port();
    let _guard = spawn_server(port);
    wait_for_server(port).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{}/api/v1/llm/providers", port))
        .json(&serde_json::json!({
            "provider": "openai",
            "apiKey": "sk-test",
            "defaultModel": "gpt-4o-test",
        }))
        .send()
        .await
        .expect("failed to POST /api/v1/llm/providers");
    assert_eq!(resp.status(), reqwest::StatusCode::CREATED);
    let body: serde_json::Value = resp.json().await.expect("POST provider body is not JSON");
    let id = body["id"].as_str().expect("POST response missing `id`").to_string();

    let resp = client
        .put(format!("http://127.0.0.1:{}/api/v1/llm/providers/{}", port, id))
        .json(&serde_json::json!({
            "apiBaseUrl": "https://llm-update.example.test/v1",
            "defaultModel": "gpt-4o-updated",
        }))
        .send()
        .await
        .expect("failed to PUT /api/v1/llm/providers/{id}");
    assert_eq!(
        resp.status(),
        reqwest::StatusCode::OK,
        "PUT /api/v1/llm/providers/{} returned {}",
        id,
        resp.status()
    );
    let body: serde_json::Value = resp.json().await.expect("PUT provider body is not JSON");
    assert_eq!(body["status"], "updated");
    // `defaultModel` must land in `model` — without the alias it would stay "gpt-4o-test".
    assert_eq!(body["model"], "gpt-4o-updated");
}
