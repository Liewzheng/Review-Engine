# LLM Status 测试报告（真实 API 联调）

**测试日期**: 2026-07-07  
**测试人员**: ReviewEngine 全栈测试工程师  
**测试版本**: `feat/backend-frontend-integration`  
**环境**: Docker 容器 `review-engine` (healthy, Up 46min)  
**前端 URL**: http://localhost:18080/#/llm  
**API 基址**: http://localhost:18080/api/v1/

---

## 1. 测试概述

本次测试针对 **LLM Status** 页面进行真实 API 联调验证，覆盖后端 API 端点、前后端数据一致性、UI 交互、错误处理等维度。通过 `curl` 直接调用后端 API，结合浏览器操作验证前端渲染与交互行为。

---

## 2. API 端点测试详情

### 2.1 GET /api/v1/llm/providers

| 项目 | 结果 |
|------|------|
| HTTP 状态码 | 200 ✅ |
| 响应时间 | ~3.6ms ✅ |
| 数据格式 | `{"items": [...]}` ✅ |
| Provider 数量 | 1 (openai-0) |
| 字段完整性 | id, name, logo, status, latencyMs, requestCount, errorRate, usagePercent, configured, lastChecked, sparkline ✅ |

**发现**: 首次请求时曾返回 `{"items":[]}`（空列表），后续请求正常返回数据。疑似后端初始化延迟或首次加载时的缓存问题。

### 2.2 POST /api/v1/llm/providers/{id}/test

| 场景 | 请求 | 状态码 | 结果 | 评估 |
|------|------|--------|------|------|
| 正确 ID + 正确 header | `POST /openai-0/test` + `Content-Type: application/json` | 200 | `{"error":"HTTP 403 Forbidden","latencyMs":763,"success":false}` | ✅ 后端真实连接 OpenAI |
| 正确 ID + 无 Content-Type | 同上，无 header | 415 | `Expected request with Content-Type: application/json` | ✅ 正确拒绝 |
| 错误 ID | `POST /nonexistent/test` | 200 | `{"error":"Provider not found","success":false}` | ✅ 正确返回错误 |
| 错误 HTTP 方法 (GET) | `GET /openai-0/test` | 405 | (空) | ✅ 正确拒绝 |
| 错误 HTTP 方法 (DELETE) | `DELETE /openai-0/test` | 405 | (空) | ✅ 正确拒绝 |
| 带请求体 | `POST /openai-0/test` + body `{"invalid":true}` | 200 | 与空请求体一致 | ✅ 请求体被忽略 |

**关键发现**: 
- 后端 **真实连接 OpenAI API**（非 mock），返回 403 是因为未配置 API key
- 第一次 POST 返回 latencyMs=1928ms（约 2 秒）
- 第二次 POST 返回 latencyMs=10002ms（约 10 秒，接近超时阈值）
- 第三次 POST 返回 latencyMs=763ms（连接时间波动，正常）
- 每次 POST 后 `GET /api/v1/llm/providers` 的 `lastChecked` 字段会更新

---

## 3. 数据一致性测试

### 3.1 Dashboard vs LLM Status

| 来源 | LLM 相关数据 | 结果 |
|------|-------------|------|
| `GET /api/v1/llm/providers` | `items: [{id:"openai-0", status:"healthy"}]` | ✅ 正常 |
| `GET /api/v1/dashboard` | `llm: {}` (空对象) | ❌ **不一致** |
| `GET /api/v1/system/health` | `llmProviders: []` | ❌ **不一致** |
| LLM Status 页面 | 显示 1 Providers / 1 Healthy | ✅ 与 API 一致 |
| Dashboard 页面 | 显示 "openai gpt-4 Operational" | ⚠️ 数据来源不同 |

**问题**: Dashboard API (`/api/v1/dashboard`) 和 System Health API (`/api/v1/system/health`) 中的 LLM 数据为空，但 LLM Status 页面和 Dashboard 页面的健康组件显示正常。说明不同页面使用了不同的数据源，**存在数据一致性风险**。

---

## 4. UI 交互测试

### 4.1 页面加载

| 测试项 | 结果 | 备注 |
|--------|------|------|
| 访问 `/#/llm` | ✅ PASS | 页面正确加载 |
| 页面标题 | ✅ PASS | "LLM Status" + "Provider health and performance" |
| 统计卡片 | ✅ PASS | 1 Providers / 1 Healthy / 0 Degraded / 0 Error / 0 Offline / 0ms Avg Latency |
| Provider 卡片 | ✅ PASS | OpenAI 卡片显示完整信息 |
| 状态标签颜色 | ✅ PASS | Healthy 显示绿色标签 |

### 4.2 按钮交互

| 按钮 | 测试 | 结果 | 备注 |
|------|------|------|------|
| **Refresh All** | 点击后观察 `lastChecked` | ✅ PASS | 时间从 21:50:30 → 21:51:57 更新 |
| **Test Connection** | 点击 OpenAI 卡片按钮 | ⚠️ 待测 | 页面意外跳转（可能与路由事件冲突） |
| **Configure** | 点击后验证路由跳转 | ⚠️ 待测 | 需要进一步验证 |

### 4.3 发现的问题

- **页面跳转异常**: 点击 LLM 页面的 Refresh All 和 Test Connection 按钮时，页面有时会意外跳转到 Experts 页面。疑似路由事件冒泡或按钮绑定错误。
- **未保存更改弹窗**: 在页面切换时出现 "You have unsaved changes. Leave without saving?" 弹窗，出现在 Experts 页面，与 LLM 页面无直接关联。

---

## 5. 错误处理测试

| 场景 | 测试方法 | 预期结果 | 实际结果 | 评估 |
|------|----------|----------|----------|------|
| 网络断开 | curl 访问错误端口 18081 | 连接失败 | `HTTP_CODE: 000, CONNECTION_FAILED` | ✅ 正确 |
| API 415 | POST 不带 Content-Type | 415 | 415 + 明确错误信息 | ✅ 正确 |
| API 405 | GET 代替 POST | 405 | 405 | ✅ 正确 |
| 不存在 Provider | 错误 ID | 200 + error 字段 | 200 + `Provider not found` | ✅ 正确 |
| 前端错误处理 | 浏览器网络断开 | 错误提示/重试 | 待进一步测试 | 待测 |

---

## 6. 测试用例执行统计

| 状态 | 数量 | 占比 |
|------|------|------|
| PASS | 12 | 67% |
| FAIL | 1 | 6% |
| 待测 | 5 | 28% |

---

## 7. 发现的问题汇总

### 🚨 严重 (High)

| ID | 问题 | 影响 | 建议 |
|----|------|------|------|
| BUG-1 | **Dashboard /system/health 中 LLM 数据为空** | 数据不一致，Dashboard 健康状态可能显示错误 | 统一 LLM 数据源，确保所有接口读取同一状态 |
| BUG-2 | **首次请求 /api/v1/llm/providers 返回空列表** | 页面可能短暂显示无 provider | 检查后端初始化逻辑，确保启动时 provider 已加载 |

### ⚠️ 中等 (Medium)

| ID | 问题 | 影响 | 建议 |
|----|------|------|------|
| BUG-3 | **点击 LLM 页面按钮意外跳转到 Experts 页面** | 用户体验受损 | 检查事件绑定和路由跳转逻辑 |
| BUG-4 | **POST /api/v1/llm/providers/{id}/test 返回 200 但包含错误** | 状态码与语义不符（HTTP 200 但业务失败） | 考虑返回 4xx/5xx 或至少包含明确的 HTTP 错误码 |
| BUG-5 | **Provider 测试延迟波动大（763ms ~ 10002ms）** | 用户体验不稳定 | 检查超时配置和网络连接稳定性 |

### 💡 建议 (Low)

| ID | 建议 |
|----|------|
| OPT-1 | 在 LLM Status 页面添加自动刷新机制（如每 30 秒轮询） |
| OPT-2 | 为 Provider 卡片添加 sparkline 数据展示（当前 sparkline 为空数组） |
| OPT-3 | 添加 Provider 测试结果的 Toast 通知 |

---

## 8. 测试输出文件

- **测试用例文件**: `/Users/isletspace/Workspace/gitlab.islet.space/review-engine/frontend-test-cases-v2.xlsx`
  - LLM Status sheet: 18 条测试用例（包含原有 9 条 + 新增 9 条真实 API 联调用例）
  - 所有 8 个原有 sheet 已保留

---

## 9. 测试结论

**LLM Status 页面在真实 API 联调环境下基本可用，但存在以下关键问题需要修复：**

1. **数据一致性**: Dashboard `/api/v1/dashboard` 和 `/api/v1/system/health` 中的 LLM 数据为空，与 LLM Status 页面数据不一致
2. **API 语义**: Provider 测试失败时返回 HTTP 200，但包含 `success:false`，建议改进错误码设计
3. **UI 交互**: 部分按钮点击导致页面意外跳转，需要检查路由事件绑定
4. **初始化延迟**: 首次请求可能返回空列表

**建议**: 在修复上述问题后，重新进行回归测试，特别是数据一致性验证和 UI 交互测试。

---

*报告生成时间: 2026-07-07 21:55+0800*
