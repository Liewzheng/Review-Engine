# System Logs Page Design

## 1. Route & Purpose

- **Route**: `/logs`
- **Purpose**: Stream and inspect system logs in a terminal-style interface. Essential for debugging review failures, configuration issues, and system errors.
- **Data**: SSE `log.entry` events append lines in real time. Historical logs fetched on mount.

## 2. Page Layout

```
PageHeader
├── Title: "System Logs" + subtitle: "Live log stream"
└── Right:
    ├── Pause / Resume button (toggle)
    ├── Download button (icon: Download)
    └── Clear button (icon: Delete)

Toolbar (margin-bottom: 12px)
├── Log level filter (ElCheckboxGroup: INFO, WARN, ERROR, DEBUG)
├── Keyword search (ElInput, prefix: Search icon)
├── Auto-scroll toggle (ElSwitch: "Auto-scroll")
└── Timestamp format (ElSelect: Relative / Absolute / ISO)

Log Terminal (full width, flex-grow)
└── LogLines container (scrollable, black background)
```

## 3. Component Breakdown

### 3.1 Toolbar

**Container**: `display: flex; flex-wrap: wrap; gap: 12px; align-items: center; padding: 12px 16px; background: var(--bg-card); border: 1px solid var(--border-color); border-radius: var(--radius-md);`

**Level filter** (`ElCheckboxGroup`):
- Options: `INFO` (blue), `WARN` (amber), `ERROR` (red), `DEBUG` (gray).
- Each checkbox has a colored dot prefix (8px circle) matching the log level color.
- Default: all checked.
- On change: filter visible log lines client-side (no re-fetch).

**Keyword search** (`ElInput`):
- Placeholder: "Filter logs..."
- Prefix icon: `ElIconSearch`.
- Clear button: `ElIconClose` on suffix when input has text.
- Debounce: `150ms` for real-time filtering.
- Filter: case-insensitive substring match on log message.

**Auto-scroll toggle** (`ElSwitch`):
- Label: "Auto-scroll".
- Default: ON.
- When ON: scroll to bottom on each new log entry.
- When OFF: maintain current scroll position; show "New logs" floating button at bottom-right when new entries arrive.

**Timestamp format** (`ElSelect`):
- Options: "Relative" (e.g., "2s ago"), "Absolute" (e.g., "14:32:05"), "ISO" (e.g., "2024-01-15T14:32:05Z").
- Default: "Relative".
- Changing format re-renders all timestamps without re-fetching.

### 3.2 Log Terminal

**Container**: `background: #0a0a0a; border: 1px solid var(--border-color); border-radius: var(--radius-md); padding: 16px; font-family: 'JetBrains Mono', monospace; font-size: 13px; line-height: 1.6; overflow-y: auto; max-height: calc(100vh - 240px);`

**Log line structure:**
```
[2024-01-15T14:32:05Z] [INFO]  Review completed for MR #123  ·  45ms
```

**Line layout** (flex row):
- Timestamp: `color: #6b7280; min-width: 180px;` (or 80px for relative)
- Level badge: `ElTag` size="small" with custom colors:
  - `INFO` → `type: "info"` (blue background, blue text)
  - `WARN` → `type: "warning"` (amber background, amber text)
  - `ERROR` → `type: "danger"` (red background, red text)
  - `DEBUG` → custom gray tag (`background: #374151; color: #9ca3af; border: none;`)
- Message: `color: #e5e7eb; flex: 1;` (light gray, almost white)
- Metadata (optional, right-aligned): `color: #6b7280; font-size: 11px;` — e.g., duration, request ID.

**Line hover**: `background: rgba(255,255,255,0.04);` (subtle highlight).
**Line selection**: `user-select: text;` — allow text selection for copying.

**Log line data interface**:
```typescript
interface LogEntry {
  id: string;           // UUID or incremental number
  timestamp: string;      // ISO 8601
  level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
  message: string;
  metadata?: {
    requestId?: string;
    durationMs?: number;
    reviewId?: string;
    expertId?: string;
  };
}
```

**Special rendering:**
- If `message` contains a review ID or MR URL, render it as a clickable `ElLink` that navigates to `/history?reviewId={id}`.
- If `level === 'ERROR'`, the entire line has a subtle left border: `border-left: 2px solid var(--error); padding-left: 6px;`.
- If `level === 'WARN'`, left border: `border-left: 2px solid var(--warning);`.

### 3.3 New Logs Floating Button

**Shown when**: auto-scroll is OFF and new logs arrive while user is scrolled up.

**Style**: `position: fixed; bottom: 24px; right: 24px; z-index: 500;`
- `ElButton` type="primary" icon `ElIconArrowDown`.
- Text: "{N} new logs" (e.g., "3 new logs").
- On click: scroll to bottom, dismiss button.
- Auto-dismiss after 10 seconds if not clicked.

### 3.4 Pause / Resume

**Pause button** (`ElButton`):
- Icon: `ElIconVideoPause` / `ElIconVideoPlay`.
- Text: "Pause" / "Resume".
- When paused: SSE connection is kept alive but incoming log entries are buffered in memory (max 1000 entries). Show "Paused — {N} buffered" indicator in toolbar.
- When resumed: flush buffered entries to the terminal and resume auto-scroll if enabled.
- Visual indicator: toolbar background changes to `rgba(245, 158, 11, 0.1)` with amber border when paused.

### 3.5 Download Logs

- Click "Download" → `GET /api/v1/logs/download?from={timestamp}&to={timestamp}&levels={levels}`.
- Generates a `.log` file download.
- Show `ElNotification` "Download started".
- Button enters loading state during request.

### 3.6 Clear Logs

- Click "Clear" → `ElMessageBox.confirm`: "Clear visible logs? This only affects the display, not stored logs."`
- On confirm: clear all rendered log lines from DOM, show empty state.
- Does NOT delete server logs.

## 4. Interactions & State Changes

### 4.1 SSE Log Stream

- Connect to `GET /api/v1/logs/stream` (SSE endpoint, separate from general events).
- Each event is a `LogEntry` JSON string.
- On receive: append to `logs` array in Pinia store, render new line.
- If auto-scroll is ON: `scrollToBottom()` after `nextTick`.
- If auto-scroll is OFF: increment "new logs" counter, show floating button.
- Buffer limit: keep only last 5000 log entries in memory. Older entries auto-discard with `shift()`.

### 4.2 Filter Changes

- Level filter or keyword change → re-filter the existing `logs` array client-side.
- No re-fetch from server.
- Show filter result count in toolbar: "Showing {N} of {M} logs".

### 4.3 Empty States

- **No logs yet**: Centered text "Waiting for logs..." with `ElIconLoading` spinner.
- **All filtered out**: Centered text "No logs match current filters" with `ElIconInfoFilled`.
- **Cleared**: Centered text "Logs cleared. New entries will appear here." with `ElIconCheck`.

## 5. Responsive Behavior

| Breakpoint | Changes |
|------------|---------|
| ≥1024px | Full toolbar inline, timestamp shown fully |
| 768–1023px | Toolbar wraps, timestamp abbreviated (HH:MM:SS) |
| <768px | Toolbar stacks vertically, log terminal full width, hide metadata column |

## 6. Animation Details

- Page enter: `page-enter` transition.
- New log line: fade in `opacity: 0 → 1`, `0.15s ease`.
- Log line removal (on clear): fade out `opacity: 1 → 0`, `0.1s ease`, then clear DOM.
- Floating button: slide up from bottom `translateY: 20px → 0`, `0.2s ease`.
- Pause overlay: `transition: background-color 0.3s ease, border-color 0.3s ease`.
- Filter change: log lines re-render with `transition: opacity 0.1s ease` (no layout animation to avoid jank on large lists).

## 7. Performance Considerations

- Use `v-for` with `:key="log.id"` on each line.
- Use `shallowRef` for log array to avoid deep reactivity overhead.
- Implement virtual scrolling if log count exceeds 2000 visible lines. Use `vue-virtual-scroller` or custom intersection-observer-based solution.
- Debounce filter re-renders at 50ms to prevent typing lag.

## 8. Data Structures

```typescript
// Pinia store: logs.ts
interface LogsState {
  logs: LogEntry[];
  filteredLogs: LogEntry[];
  levels: ('INFO' | 'WARN' | 'ERROR' | 'DEBUG')[];
  keyword: string;
  autoScroll: boolean;
  timestampFormat: 'relative' | 'absolute' | 'iso';
  isPaused: boolean;
  bufferedLogs: LogEntry[]; // while paused
  loading: boolean;
  newLogCount: number; // while scrolled up
}

// API endpoints
GET  /api/v1/logs?limit=500&levels={levels}  → LogEntry[]
GET  /api/v1/logs/stream                     → SSE (text/event-stream)
GET  /api/v1/logs/download?...               → text/plain (.log file)

// SSE event: log.entry
interface LogEntryEvent {
  entry: LogEntry;
}
```
