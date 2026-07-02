# ReviewEngine Command Reference

## Subcommand overview

| Subcommand | Purpose |
|------------|---------|
| `review` | Run a multi-expert review on a local diff, GitHub PR, or GitLab MR. |
| `repo-review` | Run a repo-wide health check across the entire codebase. |
| `describe` | Generate a summary or PR/MR description from a diff. |
| `improve` | Suggest concrete code improvements for a diff. |
| `ask` | Ask a question about the diff (requires command enablement). |
| `update_changelog` | Generate or update a changelog from recent commits. |
| `serve` | Start the REST API and webhook server. |
| `validate` | Validate a `.code-audit-config.toml` file. |
| `init` | Generate a starter config for the current project. |
| `default` | Print the built-in default config. |
| `generate-token` | Generate a random API token for `review-engine serve`. |

## Usage examples

### Repo-wide review

```bash
review-engine repo-review --local-path .
```

### Local branch review

```bash
review-engine review --local-path . --base main
```

Write Markdown to a file:

```bash
review-engine review \
  --local-path . \
  --base main \
  --format markdown \
  --output report.md
```

### Review a GitHub PR or GitLab MR

```bash
# GitHub PR
review-engine review \
  --mr-url https://github.com/owner/repo/pull/123 \
  --github-token ghp_xxx

# GitLab MR
review-engine review \
  --mr-url https://gitlab.com/owner/repo/-/merge_requests/42 \
  --gitlab-token glpat-xxx
```

Publish the report back to the PR/MR discussion:

```bash
review-engine review \
  --mr-url https://github.com/owner/repo/pull/123 \
  --github-token ghp_xxx \
  --publish
```

### Describe a PR/MR

```bash
review-engine describe --mr-url https://github.com/owner/repo/pull/123
```

### Improve a PR/MR

```bash
review-engine improve --mr-url https://gitlab.com/owner/repo/-/merge_requests/42
```

### Validate configuration

```bash
review-engine validate --config .code-audit-config.toml
```

### Start the REST / webhook server

```bash
review-engine serve --port 8080
```

Optional generated token for server authentication:

```bash
review-engine generate-token
```

### Generate a starter config

```bash
review-engine init
```

Print the built-in default config without prompts:

```bash
review-engine init --default
```

### Update changelog

```bash
review-engine update_changelog --local-path .
```
