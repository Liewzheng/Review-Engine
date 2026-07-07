# Configuration Page Design

## 1. Route & Purpose

- **Route**: `/config`
- **Purpose**: View and edit Review-Engine service settings. The page starts in read-only mode to prevent accidental changes; users must explicitly enter edit mode.
- **Data**: Fetched once on mount. Saved on explicit form submission.

## 2. Page Layout

```
PageHeader
├── Title: "Configuration" + subtitle: "Manage Review-Engine settings"
└── Right:
    ├── Edit / Save button (toggles based on mode)
    ├── Cancel button (visible only in edit mode)
    └── Refresh button (icon: Refresh)

Form Container (max-width: 900px; margin: 0 auto)
├── CardPanel: GitLab Integration
├── CardPanel: LLM Settings
├── CardPanel: Review Rules
└── CardPanel: Advanced Options
```

## 3. Component Breakdown

### 3.1 Mode Toggle

**States:**
- **Read-only**: All inputs disabled (`disabled: true`). Save button hidden. Edit button visible.
- **Edit**: All inputs enabled. Save + Cancel buttons visible. Edit button hidden.

**Edit button**: `ElButton` type="primary" icon `ElIconEdit`. Text: "Edit Configuration".
**Save button**: `ElButton` type="primary" icon `ElIconCheck`. Text: "Save Changes".
**Cancel button**: `ElButton` type="default" icon `ElIconClose`. Text: "Cancel".
**Refresh button**: `ElButton` type="default" icon `ElIconRefresh`. Text: "Refresh".

**Unsaved changes guard:**
- In edit mode, if user attempts to navigate away (Vue Router `beforeEach` guard), show `ElMessageBox.confirm`: "You have unsaved changes. Discard and leave?"
- Same confirmation on browser `beforeunload` event.

### 3.2 GitLab Integration Card

**Card title**: "GitLab Integration" + `ElIconLink` icon.
**Fields:**

| Field | Component | Type | Validation | Masked |
|-------|-----------|------|------------|--------|
| GitLab URL | `ElInput` | URL | Required, valid URL format | No |
| API Token | `ElInput` | Password | Required, min 10 chars | **Yes** (show/hide toggle) |
| Webhook Secret | `ElInput` | Password | Optional | **Yes** |
| Default Project | `ElSelect` | string | Optional | No |
| Merge Request Label | `ElInput` | string | Optional | No |
| Auto-review enabled | `ElSwitch` | boolean | — | No |

**Masked input behavior:**
- `show-password` prop on `ElInput`.
- In read-only mode: display as `••••••••••••` (12 dots) with a "Reveal" button next to it.
- Reveal button: `ElButton` size="small" icon `ElIconView`. On click, temporarily show value for 5 seconds, then auto-mask. Show countdown text: "Visible for 5s...".
- In edit mode: always show-password toggle available.

**Validation rules:**
- GitLab URL: required, must start with `http://` or `https://`.
- API Token: required, minimum 10 characters.
- Real-time validation: validate on blur. Show `ElFormItem` error message below field.

### 3.3 LLM Settings Card

**Card title**: "LLM Settings" + `ElIconCpu` icon.
**Fields:**

| Field | Component | Type | Validation | Masked |
|-------|-----------|------|------------|--------|
| Primary Provider | `ElSelect` | string | Required | No |
| OpenAI API Key | `ElInput` | Password | Required if provider is OpenAI | **Yes** |
| Anthropic API Key | `ElInput` | Password | Required if provider is Anthropic | **Yes** |
| Local Ollama URL | `ElInput` | URL | Required if provider is Ollama | No |
| Default Model | `ElSelect` | string | Required | No |
| Max Tokens | `ElInputNumber` | number | Min: 128, Max: 8192 | No |
| Temperature | `ElSlider` | number | Min: 0, Max: 2, step: 0.1 | No |
| Timeout (seconds) | `ElInputNumber` | number | Min: 5, Max: 300 | No |
| Retry Attempts | `ElInputNumber` | number | Min: 0, Max: 5 | No |

**Dynamic fields:**
- When "Primary Provider" changes, show/hide the relevant API key / URL field.
- "Default Model" options depend on selected provider. Fetched from `/api/v1/llm/models?provider={id}`.
- Temperature slider: show value label to the right (e.g., "0.7").

**Test Connection button** (inside card, right-aligned):
- `ElButton` type="default" icon `ElIconConnection`.
- On click: `POST /api/v1/llm/test` with current provider + key.
- Show inline result: green `ElTag` "Connected — 234ms" or red `ElTag` "Failed — {error message}".
- Button enters loading state during test.

### 3.4 Review Rules Card

**Card title**: "Review Rules" + `ElIconCollection` icon.
**Fields:**

| Field | Component | Type | Validation |
|-------|-----------|------|------------|
| Minimum review score | `ElSlider` | number | 0–100, step 5 |
| Block MR on critical | `ElSwitch` | boolean | — |
| Auto-comment on pass | `ElSwitch` | boolean | — |
| Comment template | `ElInput` | textarea | Max 2000 chars |
| Excluded file patterns | `ElInput` | tags | Comma-separated globs |
| Required experts | `ElCheckboxGroup` | string[] | At least 1 |
| Max review duration | `ElInputNumber` | number | Min: 30, Max: 3600 (seconds) |

**Checkbox group options** (fetched from `/api/v1/experts`):
- Security, Performance, Quality, Maintainability, Test Coverage, Documentation, Dependencies

**Excluded file patterns**: Use `ElInput` with `tag-type` (Element Plus tag input) or custom comma-separated input that renders chips. Chips are removable with `x`.

### 3.5 Advanced Options Card

**Card title**: "Advanced Options" + `ElIconTools` icon.
**Fields:**

| Field | Component | Type | Validation |
|-------|-----------|------|------------|
| Log level | `ElSelect` | string | Required |
| Log retention (days) | `ElInputNumber` | number | Min: 1, Max: 90 |
| SSE heartbeat interval | `ElInputNumber` | number | Min: 5, Max: 60 (seconds) |
| Max concurrent reviews | `ElInputNumber` | number | Min: 1, Max: 20 |
| Request timeout | `ElInputNumber` | number | Min: 10, Max: 300 (seconds) |
| Enable metrics | `ElSwitch` | boolean | — |
| Debug mode | `ElSwitch` | boolean | — |

**Card style**: collapsed by default (`ElCollapse` or `ElCard` with `v-show` toggle). Show a "Show Advanced" / "Hide Advanced" link below the Review Rules card.

## 4. Interactions & State Changes

### 4.1 Enter Edit Mode

- Click "Edit Configuration" → `isEditing = true`.
- Clone current config to `draftConfig` (deep copy).
- Enable all fields.
- Show Save and Cancel buttons.
- Page title subtitle changes to "Edit mode — remember to save your changes".

### 4.2 Save Changes

- Click "Save Changes" → validate entire form using `ElForm` `validate()` method.
- If invalid: scroll to first error, highlight field, show `ElNotification` "Please fix validation errors" (warning).
- If valid: `POST /api/v1/config` with JSON payload.
- On success:
  - Update `savedConfig` with response.
  - `isEditing = false`.
  - `ElNotification` "Configuration saved successfully" (success, 3000ms).
  - Disable all fields.
- On error:
  - `ElNotification` with server error message (error, 5000ms).
  - Keep edit mode active.

### 4.3 Cancel Edit

- Click "Cancel" → `ElMessageBox.confirm`: "Discard unsaved changes?"
- On confirm: restore `draftConfig` to `savedConfig`, `isEditing = false`, disable fields, clear dirty state.
- On cancel: remain in edit mode.

### 4.4 Real-Time Validation

- Use `ElForm` `:rules` prop with `trigger: 'blur'` for most fields.
- URL fields: custom validator using `new URL()` try/catch.
- Required-if fields: dynamic rules that check provider selection.
- Error messages: inline below field, red text, 12px.

### 4.5 Dirty State Tracking

- Compare `draftConfig` JSON string to `savedConfig` JSON string.
- If dirty: show subtle indicator dot on "Save Changes" button (red `ElBadge`).
- If not dirty: hide Save button or disable it.

## 5. Responsive Behavior

| Breakpoint | Changes |
|------------|---------|
| ≥1024px | 2-column form layout inside cards (label left, input right) |
| 768–1023px | 1-column layout, labels above inputs |
| <768px | Full-width inputs, stacked cards, sticky bottom action bar |

## 6. Animation Details

- Page enter: `page-enter` transition.
- Edit mode toggle: fade in Save/Cancel buttons (0.15s).
- Card hover: standard `CardPanel` hover border flash.
- Field enable/disable: `opacity` transition (0.15s) on disabled fields.
- Validation error: shake animation on the field container (`animation: shake 0.3s ease-in-out`).
- Save success: subtle green flash on the card border (`flash-border` animation, 0.6s).

## 7. Data Structures

```typescript
// Pinia store: config.ts
interface ConfigState {
  savedConfig: AppConfig | null;
  draftConfig: AppConfig | null;
  isEditing: boolean;
  loading: boolean;
  saving: boolean;
  dirty: boolean;
  testResults: Record<string, TestResult>; // provider -> result
}

interface AppConfig {
  gitlab: {
    url: string;
    apiToken: string;
    webhookSecret: string;
    defaultProject: string;
    mrLabel: string;
    autoReview: boolean;
  };
  llm: {
    primaryProvider: string;
    openaiApiKey: string;
    anthropicApiKey: string;
    ollamaUrl: string;
    defaultModel: string;
    maxTokens: number;
    temperature: number;
    timeoutSeconds: number;
    retryAttempts: number;
  };
  rules: {
    minScore: number;
    blockOnCritical: boolean;
    autoCommentOnPass: boolean;
    commentTemplate: string;
    excludedPatterns: string[];
    requiredExperts: string[];
    maxReviewDurationSeconds: number;
  };
  advanced: {
    logLevel: 'debug' | 'info' | 'warn' | 'error';
    logRetentionDays: number;
    sseHeartbeatInterval: number;
    maxConcurrentReviews: number;
    requestTimeout: number;
    enableMetrics: boolean;
    debugMode: boolean;
  };
}

interface TestResult {
  success: boolean;
  latencyMs?: number;
  error?: string;
  timestamp: string;
}

// API endpoints
GET  /api/v1/config              → AppConfig
POST /api/v1/config              → { success: boolean; config: AppConfig }
POST /api/v1/llm/test            → TestResult
GET  /api/v1/llm/models          → string[]
GET  /api/v1/experts             → { id: string; name: string }[]
```
