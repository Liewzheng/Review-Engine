//! Root route handler for the review-engine HTTP server.
//!
//! Provides a friendly landing page when users visit the root URL.

use axum::response::Html;

/// Returns an HTML landing page with service info and available endpoints.
pub async fn root() -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Review Engine</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; color: #333; }}
        h1 {{ color: #2563eb; }}
        .badge {{ display: inline-block; padding: 4px 12px; border-radius: 12px; font-size: 14px; font-weight: 500; }}
        .badge.ok {{ background: #dcfce7; color: #166534; }}
        .endpoint {{ background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 16px; margin: 12px 0; }}
        .endpoint code {{ background: #1e293b; color: #e2e8f0; padding: 2px 8px; border-radius: 4px; font-size: 14px; }}
        .method {{ color: #2563eb; font-weight: 600; }}
        footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #e2e8f0; color: #64748b; font-size: 14px; }}
    </style>
</head>
<body>
    <h1>🔍 Review Engine</h1>
    <p>智能代码审查服务 <span class="badge ok">运行中</span></p>

    <h2>可用端点</h2>

    <div class="endpoint">
        <span class="method">GET</span> <code>/health</code>
        <p>健康检查 — 返回服务状态</p>
    </div>

    <div class="endpoint">
        <span class="method">GET</span> <code>/metrics</code>
        <p>Prometheus 指标 — 监控数据采集</p>
    </div>

    <div class="endpoint">
        <span class="method">GET</span> <code>/progress</code>
        <p>查看所有进行中的代码审查任务</p>
    </div>

    <div class="endpoint">
        <span class="method">GET</span> <code>/progress/&#123;review_id&#125;</code>
        <p>查看指定审查任务的进度</p>
    </div>

    <div class="endpoint">
        <span class="method">POST</span> <code>/webhook/gitlab</code>
        <p>GitLab Webhook — 接收 MR 事件触发代码审查</p>
    </div>

    <div class="endpoint">
        <span class="method">POST</span> <code>/webhook/github</code>
        <p>GitHub Webhook — 接收 PR 事件触发代码审查</p>
    </div>

    <h2>API</h2>
    <div class="endpoint">
        <span class="method">GET</span> <code>/api/v1/config</code>
        <p>获取当前审查配置</p>
    </div>

    <footer>
        <p>Review Engine v{version} · <a href="https://github.com/Liewzheng/ReviewEngine">GitHub</a></p>
    </footer>
</body>
</html>"#,
        version = env!("CARGO_PKG_VERSION")
    ))
}
