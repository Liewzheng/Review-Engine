#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn bin_path() -> String {
    std::env::var("CARGO_BIN_EXE_review-engine").unwrap_or_else(|_| "target/debug/review-engine".to_string())
}

fn run(args: &[&str], current_dir: Option<&Path>) -> std::process::Output {
    let mut cmd = Command::new(bin_path());
    cmd.args(args);
    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
    }
    cmd.output().expect("failed to execute review-engine")
}

fn git_init(path: &Path) {
    run_git(path, &["init"])
}

fn git_config_user(path: &Path) {
    run_git(path, &["config", "user.email", "test@example.com"]);
    run_git(path, &["config", "user.name", "Test User"]);
}

fn git_add_and_commit(path: &Path, message: &str) {
    run_git(path, &["add", "."]);
    run_git(path, &["commit", "-m", message]);
}

fn run_git(path: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(path)
        .status()
        .expect("git is not available");
    assert!(status.success(), "git command failed: git {:?}", args);
}

#[test]
fn version() {
    let output = run(&["--version"], None);
    assert!(output.status.success(), "--version failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Review Engine v"),
        "expected version string, got: {}",
        stdout
    );
}

#[test]
fn help() {
    let output = run(&["--help"], None);
    assert!(output.status.success(), "--help failed: {:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("repo-review"), "missing repo-review subcommand");
    assert!(stdout.contains("review"), "missing review subcommand");
    assert!(stdout.contains("serve"), "missing serve subcommand");
    assert!(stdout.contains("validate"), "missing validate subcommand");
}

#[test]
fn init_default() {
    let dir = TempDir::new().unwrap();
    let output = run(&["init", "--default"], Some(dir.path()));
    assert!(output.status.success(), "init --default failed: {:?}", output);
    let config_path = dir.path().join(".code-audit-config.toml");
    assert!(config_path.exists(), ".code-audit-config.toml was not created");
}

#[test]
fn validate_valid_config() {
    let dir = TempDir::new().unwrap();
    let config = r#"
[commands]
repo_review = true
"#;
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, config).unwrap();

    let output = run(
        &["validate", "--config", config_path.to_str().unwrap()],
        Some(dir.path()),
    );
    assert!(
        output.status.success(),
        "validate failed for valid config: {:?}",
        output
    );
}

#[test]
fn validate_invalid_config() {
    let dir = TempDir::new().unwrap();
    // Overriding lead weight without disabling other default experts makes the
    // enabled experts' weights sum to more than 100.
    let config = r#"
[review_experts.lead]
weight = 99
"#;
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, config).unwrap();

    let output = run(
        &["validate", "--config", config_path.to_str().unwrap()],
        Some(dir.path()),
    );
    assert!(
        !output.status.success(),
        "validate should have failed for invalid config"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("error"),
        "stderr did not contain expected error: {}",
        stderr
    );
}

#[test]
fn repo_review_local_only() {
    let dir = TempDir::new().unwrap();
    let repo_path = dir.path();

    git_init(repo_path);
    git_config_user(repo_path);

    let src_dir = repo_path.join("src");
    std::fs::create_dir(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("main.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
    )
    .unwrap();

    let config_path = repo_path.join(".code-audit-config.toml");
    std::fs::write(
        &config_path,
        r#"
[commands]
repo_review = true
"#,
    )
    .unwrap();

    git_add_and_commit(repo_path, "initial commit");

    let report_path = repo_path.join("report.json");
    let output = run(
        &[
            "repo-review",
            "--local-path",
            ".",
            "--format",
            "json",
            "--output",
            report_path.to_str().unwrap(),
        ],
        Some(repo_path),
    );
    assert!(output.status.success(), "repo-review failed: {:?}", output);

    assert!(report_path.exists(), "report.json was not created");

    let content = std::fs::read_to_string(&report_path).unwrap();
    let value: serde_json::Value = serde_json::from_str(&content).expect("report.json is not valid JSON");

    // The report should contain scoring, summary, and expert information.
    assert!(value.get("overview").is_some(), "missing overview");
    assert!(value["overview"].get("health_score").is_some(), "missing health_score");
    assert!(value.get("expert_scores").is_some(), "missing expert_scores");
    assert!(value.get("conclusion").is_some(), "missing conclusion");
}
