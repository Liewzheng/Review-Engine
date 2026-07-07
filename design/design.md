# Review-Engine Dashboard — Global Design System

## 1. Design Philosophy

- **Developer-first**: dense information, minimal chrome, keyboard-friendly.
- **Dark by default**: the UI opens in dark mode; light mode is an opt-in toggle.
- **Status at a glance**: color-coded badges, progress bars, and health dots surface system state without reading.
- **Performance**: skeleton screens over spinners; SSE-driven partial updates instead of full page reloads.

## 2. Route Table

| Route Path        | Page Name          | Description                              |
|-------------------|--------------------|------------------------------------------|
| `/` or `/dashboard` | Dashboard          | Overview KPIs, 24h trend, recent activity, health |
| `/history`        | Review History     | Paginated, filterable, searchable review log |
| `/config`         | Configuration      | Read-only / editable service settings     |
| `/queue`          | Queue Monitor      | Real-time task queue with SSE updates     |
| `/llm`            | LLM Status         | Provider health cards and test connections |
| `/logs`           | System Logs        | Terminal-style streaming log viewer       |
| `/experts`        | Experts Management | Enable / disable / inspect LLM experts    |

> All routes use **Vue Router HashRouter** (`#/`). No server-side routing required.

## 3. CSS Variable Design Tokens

### 3.1 Dark Mode (Default)

```css
:root {
  --bg-primary: #0f172a;        /* slate-900  — page background */
  --bg-surface: #1e293b;        /* slate-800  — sidebar, header, cards */
  --bg-card: #334155;           /* slate-700  — elevated panels */
  --border-color: #475569;      /* slate-600  — dividers, borders */
  --text-primary: #f1f5f9;      /* slate-100  — headings, primary text */
  --text-secondary: #94a3b8;    /* slate-400  — labels, captions */
  --brand: #6366f1;             /* indigo-500 — primary actions, active nav */
  --brand-hover: #4f46e5;       /* indigo-600 — hover state */
  --success: #22c55e;           /* green-500  */
  --warning: #f59e0b;           /* amber-500  */
  --error: #ef4444;             /* red-500    */
  --info: #3b82f6;              /* blue-500   */
  --offline: #6b7280;           /* gray-500   */

  /* Derived */
  --bg-hover: rgba(255,255,255,0.04);
  --bg-active: rgba(99,102,241,0.12);
  --shadow-card: 0 4px 6px -1px rgba(0,0,0,0.4), 0 2px 4px -1px rgba(0,0,0,0.3);
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 12px;
}
```

### 3.2 Light Mode

```css
[data-theme="light"] {
  --bg-primary: #f8fafc;        /* slate-50   */
  --bg-surface: #ffffff;        /* white      */
  --bg-card: #f1f5f9;           /* slate-100  */
  --border-color: #cbd5e1;      /* slate-300  */
  --text-primary: #0f172a;      /* slate-900  */
  --text-secondary: #64748b;  /* slate-500  */
  --brand: #6366f1;
  --brand-hover: #4f46e5;
  --success: #16a34a;           /* green-600  */
  --warning: #d97706;           /* amber-600  */
  --error: #dc2626;             /* red-600    */
  --info: #2563eb;              /* blue-600   */
  --offline: #9ca3af;           /* gray-400   */

  --bg-hover: rgba(0,0,0,0.03);
  --bg-active: rgba(99,102,241,0.08);
  --shadow-card: 0 4px 6px -1px rgba(0,0,0,0.1), 0 2px 4px -1px rgba(0,0,0,0.06);
}
```

### 3.3 Auto-Adaptation

```css
@media (prefers-color-scheme: light) {
  :root:not([data-theme="dark"]) {
    /* copy light mode values here or import the block above */
  }
}
```

The app stores the explicit user choice in `localStorage` under key `review-engine-theme`. If no key exists, follow `prefers-color-scheme`.

## 4. Typography

| Role | Font | Weight | Size | Line-height | Letter-spacing |
|------|------|--------|------|-------------|----------------|
| Page title | Inter | 600 | 24px | 1.3 | -0.02em |
| Section heading | Inter | 600 | 16px | 1.4 | -0.01em |
| Card title | Inter | 500 | 14px | 1.4 | 0 |
| Body | Inter | 400 | 14px | 1.5 | 0 |
| Caption / Label | Inter | 400 | 12px | 1.4 | 0.01em |
| Monospace data | JetBrains Mono | 400 | 13px | 1.5 | 0 |
| Monospace small | JetBrains Mono | 400 | 11px | 1.4 | 0 |

Font imports (Google Fonts):
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
```

## 5. Layout System

### 5.1 App Shell

```
┌──────────────────────────────────────────────┐
│  TopBar (56px, fixed)                         │
├────────┬─────────────────────────────────────┤
│        │                                      │
│ Sidebar│  Main Content                        │
│ (200px)│  (flexible, max-width: 1400px)       │
│ fixed  │  padding: 24px                       │
│        │                                      │
└────────┴─────────────────────────────────────┘
```

- **TopBar**: `position: fixed; top: 0; left: 0; right: 0; height: 56px; z-index: 1000;`
- **Sidebar**: `position: fixed; top: 56px; left: 0; width: 200px; bottom: 0; z-index: 999;`
- **Main**: `margin-left: 200px; margin-top: 56px; min-height: calc(100vh - 56px);`
- **Content wrapper**: `max-width: 1400px; margin: 0 auto; padding: 24px;`

### 5.2 Sidebar Collapse

- Toggle button in TopBar (hamburger icon).
- Collapsed width: `64px` (icons only, labels hidden).
- Transition: `width 0.25s cubic-bezier(0.4, 0, 0.2, 1)`.
- On collapse, show tooltips on hover for nav items.
- Main content margin-left adjusts accordingly.

### 5.3 Mobile (< 768px)

- Sidebar hidden by default; overlay drawer on hamburger click.
- Drawer width: `240px`; backdrop: `rgba(0,0,0,0.5)`.
- Main content margin-left: `0`.
- Tables switch to horizontal scroll; cards stack vertically.

## 6. Shared Components

### 6.1 StatusBadge

A reusable status indicator using Element Plus `ElTag` with custom color mapping.

**Props:**
- `status: 'success' | 'warning' | 'error' | 'info' | 'offline' | 'running' | 'queued' | 'failed'`
- `size: 'small' | 'default'` (default: `small`)
- `dotOnly: boolean` (default: `false`) — show only the colored dot, no text

**Color mapping:**
| Status | Tag Type | Effect |
|--------|----------|--------|
| success | success | green |
| running | success | green + `ElIconLoading` spinner prefix |
| warning | warning | amber |
| queued | info | blue |
| error | danger | red |
| failed | danger | red |
| info | info | blue |
| offline | info | gray (custom style override) |

**Dot-only variant:**
- Render a `<span>` with `width: 8px; height: 8px; border-radius: 50%;` using the status color.
- Add `animation: pulse 2s infinite` when status is `running`.

```css
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}
```

### 6.2 PageHeader

**Structure:**
- Left: page title (24px, font-weight 600) + optional subtitle (14px, `--text-secondary`)
- Right: action buttons (refresh, settings, etc.) aligned with `ElButtonGroup`

**Spacing:**
- Margin bottom: `24px` between PageHeader and content.
- `display: flex; justify-content: space-between; align-items: center;`

### 6.3 CardPanel

A wrapper around Element Plus `ElCard` with design-system overrides.

**Overrides:**
- `background: var(--bg-card)`
- `border: 1px solid var(--border-color)`
- `border-radius: var(--radius-md)`
- `box-shadow: var(--shadow-card)`
- Header: `font-size: 14px; font-weight: 500; color: var(--text-primary); padding: 16px 20px; border-bottom: 1px solid var(--border-color);`
- Body: `padding: 20px;`

### 6.4 DataTable

A wrapper around `ElTable` with design-system defaults.

**Defaults:**
- `stripe: false` (no zebra striping; use hover instead)
- `border: false`
- `header-cell-style`: `background: var(--bg-surface); color: var(--text-secondary); font-weight: 500; font-size: 12px; text-transform: uppercase; letter-spacing: 0.05em;`
- `cell-style`: `padding: 10px 12px; font-size: 13px; color: var(--text-primary);`
- `row-hover`: `background: var(--bg-hover)`
- `highlight-current-row: false`

### 6.5 KpiCard

A fixed-height stat card used in Dashboard and other overview pages.

**Structure:**
```
┌────────────────────────┐
│ Icon  Title            │  → top row, icon 24px, title 12px secondary
│                        │
│ 1,234                  │  → value, 28px, font-weight 600, primary color
│ ↑ 12% vs last week     │  → trend, 12px, green/red based on direction
└────────────────────────┘
```

**Dimensions:**
- `min-height: 120px; padding: 20px;`
- Icon container: `width: 40px; height: 40px; border-radius: var(--radius-sm); background: rgba(var(--brand-rgb), 0.1); display: flex; align-items: center; justify-content: center;`
- Icon size: `20px; color: var(--brand);`

### 6.6 SkeletonLoader

Use `ElSkeleton` with custom styling:
- `background: var(--bg-hover)`
- `animated: true`
- Rows: 3–5 depending on content area.
- For table skeletons: render 5 rows of `ElSkeletonItem` with `variant="text"`.

## 7. Animation Specifications

### 7.1 Page Transition

When navigating between routes, apply a fade + slight slide:
```css
.page-enter-active,
.page-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.page-enter-from {
  opacity: 0;
  transform: translateY(6px);
}
.page-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
```

Use Vue Router `<Transition name="page">` wrapper around `<RouterView>`.

### 7.2 Card Hover

```css
.card-panel:hover {
  border-color: var(--brand);
  box-shadow: 0 0 0 1px var(--brand), var(--shadow-card);
  transition: all 0.2s ease;
}
```

### 7.3 Table Row Hover

- `transition: background-color 0.1s ease;`

### 7.4 Toast / Notification

- Use Element Plus `ElNotification`.
- Position: `top-right`.
- Duration: `3000ms` for success, `5000ms` for error.
- Success: `type: 'success'` (green).
- Error: `type: 'error'` (red) + `duration: 0` for critical errors requiring manual dismissal.

### 7.5 SSE Update Flash

When a row or card receives an SSE update, briefly flash the border:
```css
@keyframes flash-border {
  0%   { border-color: var(--brand); box-shadow: 0 0 0 2px rgba(99,102,241,0.3); }
  100% { border-color: var(--border-color); box-shadow: none; }
}
.sse-update {
  animation: flash-border 0.6s ease-out;
}
```

## 8. Dependencies & Versions

| Package | Version | Purpose |
|---------|---------|---------|
| vue | ^3.4 | Framework |
| vue-router | ^4.3 | HashRouter routing |
| pinia | ^2.1 | State management |
| element-plus | ^2.6 | UI component library |
| @element-plus/icons-vue | ^2.3 | Icon set |
| axios | ^1.6 | HTTP client |
| lightweight-charts | ^4.1 | 24h trend chart (Dashboard) |
| vite | ^5.2 | Build tool |
| typescript | ^5.4 | Type safety |
| @vitejs/plugin-vue | ^5.0 | Vue SFC support |

> **Note**: `lightweight-charts` is used only for the 24h trend line chart on Dashboard. All other charts (progress bars, mini sparklines) are implemented with CSS or SVG.

## 9. Asset Manifest

### 9.1 Icons (Element Plus Icons)

| Name | Icon Component | Usage |
|------|----------------|-------|
| Home | `ElIconHomeFilled` | Dashboard nav |
| Document | `ElIconDocument` | History nav |
| Setting | `ElIconSetting` | Config nav |
| Refresh | `ElIconRefresh` | Queue nav |
| CPU | `ElIconCpu` | LLM nav |
| List | `ElIconList` | Logs nav |
| User | `ElIconUserFilled` | Experts nav |
| Moon | `ElIconMoon` | Dark mode toggle |
| Sunny | `ElIconSunny` | Light mode toggle |
| Search | `ElIconSearch` | Search inputs |
| Filter | `ElIconFilter` | Filter buttons |
| Close | `ElIconClose` | Modal close / clear |
| Check | `ElIconCheck` | Success actions |
| Warning | `ElIconWarning` | Alerts |
| Info | `ElIconInfoFilled` | Info tooltips |
| ArrowRight | `ElIconArrowRight` | Detail links |
| More | `ElIconMore` | Action menus |
| Loading | `ElIconLoading` | Spinners |
| Fold | `ElIconFold` | Collapse sidebar |
| Expand | `ElIconExpand` | Expand sidebar |

### 9.2 Custom SVG Icons (Inline or Asset Files)

| Icon | File / SVG | Usage |
|------|------------|-------|
| GitLab logo | `gitlab.svg` | GitLab integration status |
| GitHub logo | `github.svg` | GitHub integration status |
| OpenAI logo | `openai.svg` | LLM provider card |
| Anthropic logo | `anthropic.svg` | LLM provider card |
| Security shield | `shield.svg` | Security expert icon |
| Performance bolt | `bolt.svg` | Performance expert icon |
| Quality check | `check-circle.svg` | Quality expert icon |

All custom SVGs are placed in `src/assets/icons/` and imported as Vue components via `vite-plugin-svg-icons` or inline `<svg>`.

## 10. API Conventions

- Base URL: read from `VITE_API_BASE_URL` env variable (default: `/api/v1`).
- All endpoints return JSON with shape: `{ success: boolean; data?: T; error?: string; }`.
- HTTP 401 → redirect to login (if auth added later).
- HTTP 500 → show `ElNotification` with error message and "Retry" action.
- Loading state: managed per-component via Pinia or local `ref<boolean>`. Never block UI globally.

## 11. Pinia Store Structure

```
stores/
  ├── layout.ts      — sidebar collapse, theme, mobile drawer
  ├── dashboard.ts   — KPIs, trend data, recent activity, health
  ├── history.ts     — review list, filters, pagination
  ├── config.ts      — settings, dirty state, validation
  ├── queue.ts       — queue tasks, SSE connection, stats
  ├── llm.ts         — provider status, health results
  ├── logs.ts        — log entries, filters, pause state
  └── experts.ts     — expert list, enable/disable
```

## 12. SSE (EventSource) Conventions

- Endpoint: `GET /api/v1/events` — server-sent events stream.
- Event types:
  - `queue.update` — task progress, status change
  - `llm.status` — provider health update
  - `log.entry` — new log line
  - `system.health` — overall health change
- Reconnect: auto-reconnect with exponential backoff (max 30s).
- On disconnect: show offline warning in TopBar; reconnecting spinner.
- Store connection status in `layout.ts` (Pinia).

## 13. Accessibility

- All interactive elements have `cursor: pointer`.
- Focus rings: `outline: 2px solid var(--brand); outline-offset: 2px;` for keyboard navigation.
- Color alone is never the only status indicator; use icons + text.
- Minimum contrast ratio: 4.5:1 for body text, 3:1 for UI chrome.
- `aria-label` on icon-only buttons.
- Table headers have `scope="col"`.
