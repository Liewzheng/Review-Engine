use axum::{http::StatusCode, Json};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;

use super::dispatcher::MrDispatcher;
use super::webhook::WebhookHandler;

use async_trait::async_trait;
use axum::http::HeaderMap;

type HmacSha256 = Hmac<Sha256>;

/// GitLab webhook handler.
///
/// Supports two verification methods (can be configured independently or together):
/// 1. **Secret token** (`X-Gitlab-Token` header) — legacy, configured via `webhook_secret`.
/// 2. **Signing token** (`X-Gitlab-Webhook-Signature` header, HMAC-SHA256 of body) —
///    GitLab 19.0+, configured via `signing_secret`.
///
/// See: <https://docs.gitlab.com/19.0/user/project/integrations/webhooks/#signing-tokens>
#[derive(Clone)]
pub struct GitLabWebhookHandler {
    pub webhook_secret: String,
    pub signing_secret: Option<String>,
    pub dispatcher: MrDispatcher,
    pub token: String,
}

impl GitLabWebhookHandler {
    /// Create a new GitLab webhook handler.
    ///
    /// `webhook_secret` — legacy `X-Gitlab-Token` verification (empty string disables).
    /// `signing_secret` — HMAC-SHA256 body signature verification (`None` disables).
    pub fn new(
        webhook_secret: String,
        signing_secret: Option<String>,
        dispatcher: MrDispatcher,
        token: String,
    ) -> Self {
        Self {
            webhook_secret,
            signing_secret,
            dispatcher,
            token,
        }
    }

    /// Verify `X-Gitlab-Webhook-Signature` header (HMAC-SHA256 of body).
    fn verify_signing(&self, headers: &HeaderMap, body: &str) -> Result<(), (StatusCode, Json<Value>)> {
        let secret = match &self.signing_secret {
            Some(s) if !s.is_empty() => s,
            _ => return Ok(()), // signing not configured — skip this check
        };

        let signature_raw = headers
            .get("X-Gitlab-Webhook-Signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if signature_raw.is_empty() {
            tracing::warn!("GitLab webhook signing secret configured but X-Gitlab-Webhook-Signature header missing");
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "missing signing signature header"})),
            ));
        }

        let signature = if let Some(s) = signature_raw.strip_prefix("sha256=") {
            s
        } else {
            tracing::warn!("GitLab webhook signing signature does not start with sha256=");
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "invalid signing signature format"})),
            ));
        };

        if verify_signature(secret, body, signature) {
            Ok(())
        } else {
            tracing::warn!("GitLab webhook HMAC signing signature mismatch — check GITLAB_WEBHOOK_SIGNING_SECRET");
            Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "invalid signing signature"})),
            ))
        }
    }

    /// Verify legacy `X-Gitlab-Token` header.
    fn verify_secret_token(&self, headers: &HeaderMap) -> Result<(), (StatusCode, Json<Value>)> {
        if self.webhook_secret.is_empty() {
            return Ok(()); // legacy secret not configured — skip this check
        }

        let token = headers
            .get("X-Gitlab-Token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token_bytes = token.as_bytes();
        let secret_bytes = self.webhook_secret.as_bytes();
        if token_bytes.len() != secret_bytes.len() {
            tracing::warn!("GitLab webhook received with invalid token");
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "invalid token"})),
            ));
        }
        if !bool::from(subtle::ConstantTimeEq::ct_eq(token_bytes, secret_bytes)) {
            tracing::warn!("GitLab webhook received with invalid token");
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "invalid token"})),
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl WebhookHandler for GitLabWebhookHandler {
    fn path(&self) -> &'static str {
        "/webhook/gitlab"
    }

    fn name(&self) -> &'static str {
        "gitlab"
    }

    async fn verify(&self, headers: &HeaderMap, body: &str) -> Result<(), (StatusCode, Json<Value>)> {
        // At least one verification method must be configured; reject all otherwise
        if self.webhook_secret.is_empty() && self.signing_secret.is_none() {
            tracing::warn!("GitLab webhook rejected: neither webhook_secret nor signing_secret configured");
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "no verification configured"})),
            ));
        }

        self.verify_secret_token(headers)?;
        self.verify_signing(headers, body)?;
        Ok(())
    }

    async fn handle_event(&self, headers: &HeaderMap, body: &str) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
        let event = headers
            .get("X-Gitlab-Event")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        match event {
            "Merge Request Hook" => handle_mr_hook(&body, &self.dispatcher, &self.token)
                .await
                .map_err(|status| (status, Json(serde_json::json!({"error": "request failed"})))),
            "Note Hook" => handle_note_hook(&body, &self.dispatcher, &self.token)
                .await
                .map_err(|status| (status, Json(serde_json::json!({"error": "request failed"})))),
            "Push Hook" => handle_push_hook(&body)
                .await
                .map_err(|status| (status, Json(serde_json::json!({"error": "request failed"})))),
            _ => {
                tracing::debug!("Ignoring unsupported GitLab event: {}", event);
                Ok(Json(serde_json::json!({ "status": "ignored" })))
            }
        }
    }
}

/// Parsed payload from a GitLab Merge Request webhook event.
struct MrHookPayload {
    action: String,
    mr_url: String,
    mr_iid: u64,
    sha: String,
    gitlab_token: String,
}

/// Parse and validate an MR webhook body into its essential fields.
fn parse_mr_hook_payload(body: &str, gitlab_token: &str) -> Result<MrHookPayload, StatusCode> {
    let parsed: Value = serde_json::from_str(body).map_err(|e| {
        tracing::error!("Failed to parse MR hook: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let action = parsed["object_attributes"]["action"].as_str().unwrap_or("").to_string();
    let project_url = parsed["project"]["web_url"].as_str().unwrap_or("").to_string();
    let mr_iid = parsed["object_attributes"]["iid"].as_u64().unwrap_or(0);
    let mr_url = if !project_url.is_empty() && mr_iid > 0 {
        format!("{}/-/merge_requests/{}", project_url, mr_iid)
    } else {
        String::new()
    };
    let sha = parsed["object_attributes"]["last_commit"]["id"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(MrHookPayload {
        action,
        mr_url,
        mr_iid,
        sha,
        gitlab_token: gitlab_token.to_string(),
    })
}

/// Spawn a background task that runs the full review for an MR.
fn spawn_mr_review_task(dispatcher: &MrDispatcher, mr_url: String, sha: String, gitlab_token: String, mr_iid: u64) {
    let d = dispatcher.clone();
    tokio::spawn(async move {
        if let Err(e) = run_review_for_mr(&mr_url, &gitlab_token, Some(&d), Some(&mr_url), Some(&sha)).await {
            tracing::error!("Review failed for MR !{}: {:?}", mr_iid, e);
            d.reset(&mr_url).await;
        }
    });
}

/// Handle the `InProgress` dispatcher state: wait and then retry.
async fn handle_mr_in_progress(dispatcher: &MrDispatcher, mr_url: &str, sha: &str, gitlab_token: &str, mr_iid: u64) {
    tracing::info!("MR !{} review in progress, waiting...", mr_iid);
    dispatcher.wait(mr_url).await;
    // After wait, re-check if current SHA needs a new review
    match dispatcher.try_start(mr_url, sha).await {
        super::dispatcher::ShouldStart::Go => {
            spawn_mr_review_task(
                dispatcher,
                mr_url.to_string(),
                sha.to_string(),
                gitlab_token.to_string(),
                mr_iid,
            );
        }
        _ => {
            tracing::info!("No new review needed for MR !{} after wait", mr_iid);
        }
    }
}

/// Dispatch an MR webhook event to start or defer a review based on the
/// dispatcher state.
async fn dispatch_mr_event(dispatcher: &MrDispatcher, mr_url: &str, sha: &str, gitlab_token: &str, mr_iid: u64) {
    match dispatcher.try_start(mr_url, sha).await {
        super::dispatcher::ShouldStart::Go => {
            spawn_mr_review_task(
                dispatcher,
                mr_url.to_string(),
                sha.to_string(),
                gitlab_token.to_string(),
                mr_iid,
            );
        }
        super::dispatcher::ShouldStart::AlreadyReviewed => {
            tracing::info!("Skipping MR !{}: already reviewed at SHA {}", mr_iid, sha);
        }
        super::dispatcher::ShouldStart::InProgress => {
            handle_mr_in_progress(dispatcher, mr_url, sha, gitlab_token, mr_iid).await;
        }
    }
}

async fn handle_mr_hook(body: &str, dispatcher: &MrDispatcher, gitlab_token: &str) -> Result<Json<Value>, StatusCode> {
    let payload = parse_mr_hook_payload(body, gitlab_token)?;

    tracing::info!("MR !{} webhook received: action={}", payload.mr_iid, payload.action);

    // Only process opened/reopened/updated MRs
    if payload.action == "open" || payload.action == "reopen" || payload.action == "update" {
        if payload.mr_url.is_empty() || payload.gitlab_token.is_empty() {
            tracing::warn!("Skipping review: missing MR URL or GITLAB_TOKEN");
            return Ok(Json(serde_json::json!({
                "status": "skipped",
                "reason": "missing MR URL or GITLAB_TOKEN"
            })));
        }

        if payload.sha.is_empty() {
            tracing::warn!("Skipping review: missing commit SHA");
            return Ok(Json(serde_json::json!({
                "status": "skipped",
                "reason": "missing commit SHA"
            })));
        }

        dispatch_mr_event(
            dispatcher,
            &payload.mr_url,
            &payload.sha,
            &payload.gitlab_token,
            payload.mr_iid,
        )
        .await;
    }

    Ok(Json(serde_json::json!({
        "status": "received",
        "action": payload.action,
    })))
}

/// Run a full review for an MR and publish results.
async fn run_review_for_mr(
    mr_url: &str,
    gitlab_token: &str,
    dispatcher: Option<&MrDispatcher>,
    dispatch_key: Option<&str>,
    sha: Option<&str>,
) -> anyhow::Result<()> {
    super::run_review_common(mr_url, gitlab_token, dispatcher, dispatch_key, sha).await
}

async fn handle_note_hook(
    body: &str,
    dispatcher: &MrDispatcher,
    gitlab_token: &str,
) -> Result<Json<Value>, StatusCode> {
    let parsed: Value = serde_json::from_str(body).map_err(|e| {
        tracing::error!("Failed to parse Note hook: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let note = parsed["object_attributes"]["note"].as_str().unwrap_or("");
    let note_lower = note.to_lowercase();

    // Check for commands like /review, /describe
    if note_lower.starts_with("/review") || note_lower.starts_with("/describe") {
        let project_url = parsed["project"]["web_url"].as_str().unwrap_or("").to_string();
        let mr_iid = parsed["merge_request"]["iid"]
            .as_u64()
            .or_else(|| parsed["object_attributes"]["noteable_iid"].as_u64())
            .unwrap_or(0);
        let mr_url = if !project_url.is_empty() && mr_iid > 0 {
            format!("{}/-/merge_requests/{}", project_url, mr_iid)
        } else {
            String::new()
        };

        if !mr_url.is_empty() && !gitlab_token.is_empty() {
            let url = mr_url;
            let token = gitlab_token.to_string();
            let sha = format!("note_{}", uuid::Uuid::new_v4());

            match dispatcher.try_start(&url, &sha).await {
                super::dispatcher::ShouldStart::Go => {
                    let d = dispatcher.clone();
                    let u = url;
                    let s = sha;
                    tokio::spawn(async move {
                        if let Err(e) = run_review_for_mr(&u, &token, Some(&d), Some(&u), Some(&s)).await {
                            tracing::error!("Review from note failed: {:?}", e);
                            d.reset(&u).await;
                        }
                    });
                }
                _ => {
                    tracing::info!("Note review skipped or already in progress");
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "status": "received",
        "note_preview": &note[..note.len().min(100)],
    })))
}

async fn handle_push_hook(body: &str) -> Result<Json<Value>, StatusCode> {
    let _parsed: Value = serde_json::from_str(body).map_err(|e| {
        tracing::error!("Failed to parse Push hook: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    tracing::info!("Push hook received");

    Ok(Json(serde_json::json!({
        "status": "received",
    })))
}

/// Verify the HMAC-SHA256 signature (shared with GitHub handler).
fn verify_signature(secret: &str, body: &str, signature: &str) -> bool {
    let decoded = match hex::decode(signature) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body.as_bytes());
    mac.verify_slice(&decoded).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_handler_creation() {
        let handler = GitLabWebhookHandler::new(
            "test-secret".to_string(),
            Some("test-signing".to_string()),
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        assert_eq!(handler.webhook_secret, "test-secret");
        assert_eq!(handler.signing_secret, Some("test-signing".to_string()));
        assert_eq!(handler.path(), "/webhook/gitlab");
        assert_eq!(handler.name(), "gitlab");
    }

    #[test]
    fn test_webhook_handler_empty_secret() {
        let handler = GitLabWebhookHandler::new(String::new(), None, MrDispatcher::new(), "test-token".to_string());
        assert!(handler.webhook_secret.is_empty());
        assert!(handler.signing_secret.is_none());
    }

    // ── Legacy X-Gitlab-Token tests ────────────────────────────────────

    #[tokio::test]
    async fn test_webhook_verify_valid_token() {
        let handler = GitLabWebhookHandler::new(
            "my-secret".to_string(),
            None,
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        let mut headers = HeaderMap::new();
        headers.insert("X-Gitlab-Token", "my-secret".parse().unwrap());
        let result = handler.verify(&headers, "").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_webhook_verify_invalid_token() {
        let handler = GitLabWebhookHandler::new(
            "my-secret".to_string(),
            None,
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        let mut headers = HeaderMap::new();
        headers.insert("X-Gitlab-Token", "wrong-secret".parse().unwrap());
        let result = handler.verify(&headers, "").await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_webhook_verify_missing_token() {
        let handler = GitLabWebhookHandler::new(
            "my-secret".to_string(),
            None,
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        let headers = HeaderMap::new();
        let result = handler.verify(&headers, "").await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_webhook_verify_empty_secret_rejects_any_token() {
        let handler = GitLabWebhookHandler::new(String::new(), None, MrDispatcher::new(), "test-token".to_string());
        let mut headers = HeaderMap::new();
        headers.insert("X-Gitlab-Token", "anything".parse().unwrap());
        let result = handler.verify(&headers, "").await;
        // Empty secret and no signing_secret → no verification configured → rejected
        assert!(result.is_err());
    }

    // ── Signing token (X-Gitlab-Webhook-Signature) tests ───────────────

    #[tokio::test]
    async fn test_signing_verify_valid_signature() {
        let secret = "my-signing-secret";
        let body = r#"{"object_attributes":{"action":"open","iid":1}}"#;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        let sig = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        let handler = GitLabWebhookHandler::new(
            String::new(),
            Some(secret.to_string()),
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        let mut headers = HeaderMap::new();
        headers.insert("X-Gitlab-Webhook-Signature", sig.parse().unwrap());
        let result = handler.verify(&headers, body).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_signing_verify_invalid_signature() {
        let handler = GitLabWebhookHandler::new(
            String::new(),
            Some("my-signing-secret".to_string()),
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Gitlab-Webhook-Signature",
            "sha256=0000000000000000000000000000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
        );
        let result = handler.verify(&headers, "body").await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_signing_verify_missing_signature() {
        let handler = GitLabWebhookHandler::new(
            String::new(),
            Some("my-signing-secret".to_string()),
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        let headers = HeaderMap::new();
        let result = handler.verify(&headers, "body").await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_signing_verify_no_prefix_rejected() {
        let handler = GitLabWebhookHandler::new(
            String::new(),
            Some("my-signing-secret".to_string()),
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        let mut headers = HeaderMap::new();
        headers.insert("X-Gitlab-Webhook-Signature", "abcdef123456".parse().unwrap());
        let result = handler.verify(&headers, "body").await;
        assert!(result.is_err());
        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    // ── Both methods configured ─────────────────────────────────────────

    #[tokio::test]
    async fn test_both_verify_all_pass() {
        let secret = "my-signing-secret";
        let body = r#"{"object_attributes":{"action":"open"}}"#;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        let sig = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        let handler = GitLabWebhookHandler::new(
            "my-webhook-secret".to_string(),
            Some(secret.to_string()),
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        let mut headers = HeaderMap::new();
        headers.insert("X-Gitlab-Token", "my-webhook-secret".parse().unwrap());
        headers.insert("X-Gitlab-Webhook-Signature", sig.parse().unwrap());
        let result = handler.verify(&headers, body).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_both_verify_token_wrong() {
        let secret = "my-signing-secret";
        let body = "body";
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        let sig = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        let handler = GitLabWebhookHandler::new(
            "my-webhook-secret".to_string(),
            Some(secret.to_string()),
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        let mut headers = HeaderMap::new();
        headers.insert("X-Gitlab-Token", "wrong-secret".parse().unwrap());
        headers.insert("X-Gitlab-Webhook-Signature", sig.parse().unwrap());
        let result = handler.verify(&headers, body).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_both_verify_signing_wrong() {
        let handler = GitLabWebhookHandler::new(
            "my-webhook-secret".to_string(),
            Some("my-signing-secret".to_string()),
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        let mut headers = HeaderMap::new();
        headers.insert("X-Gitlab-Token", "my-webhook-secret".parse().unwrap());
        headers.insert(
            "X-Gitlab-Webhook-Signature",
            "sha256=0000000000000000000000000000000000000000000000000000000000000000"
                .parse()
                .unwrap(),
        );
        let result = handler.verify(&headers, "body").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_signing_not_configured_skipped() {
        let handler = GitLabWebhookHandler::new(
            "my-webhook-secret".to_string(),
            None,
            MrDispatcher::new(),
            "test-token".to_string(),
        );
        let mut headers = HeaderMap::new();
        headers.insert("X-Gitlab-Token", "my-webhook-secret".parse().unwrap());
        // No X-Gitlab-Webhook-Signature header, but signing not configured → OK
        let result = handler.verify(&headers, "body").await;
        assert!(result.is_ok());
    }

    // ── verify_signature helper tests ──────────────────────────────────

    #[test]
    fn test_verify_signature_valid() {
        let secret = "my-secret";
        let body = r#"{"action":"opened"}"#;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());
        assert!(verify_signature(secret, body, &expected));
    }

    #[test]
    fn test_verify_signature_wrong_secret() {
        let body = r#"{"action":"opened"}"#;
        let mut mac = HmacSha256::new_from_slice(b"other-secret").unwrap();
        mac.update(body.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());
        assert!(!verify_signature("my-secret", body, &sig));
    }

    #[test]
    fn test_verify_signature_invalid_hex() {
        assert!(!verify_signature("secret", "body", "not-hex"));
    }

    #[test]
    fn test_verify_signature_empty_secret() {
        let body = "test";
        assert!(!verify_signature("", body, "abc123"));
    }

    #[test]
    fn test_verify_signature_tampered_body() {
        let secret = "my-secret";
        let body = r#"{"action":"opened"}"#;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());
        assert!(!verify_signature(secret, r#"{"action":"closed"}"#, &sig));
    }
}
