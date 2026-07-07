# Queue Monitor Page Design

## 1. Route & Purpose

- **Route**: `/queue`
- **Purpose**: Real-time monitoring of the review task queue. The primary operational view for DevOps engineers to observe, pause, and manage review throughput.
- **Data**: Heavily SSE-driven. Initial state on mount, then live updates via `queue.update` events.

## 2. Page Layout

```
PageHeader
├── Title: "Queue Monitor" + subtitle: "Real-time review task queue"
└── Right:
    ├── Pause / Resume Queue button (toggle)
    ├── Cancel All Failed button
    └── Refresh button

Stats Row (margin-bottom: 16px)
├── StatCard: Active Tasks
├── StatCard: Queued Tasks
├── StatCard: Failed Tasks
└── StatCard: Queue Depth

Task Cards Grid (margin-top: 16px)
├── Active Tasks section
│   └── Grid of TaskCard (running tasks)
├── Queued Tasks section
│   └── Grid of TaskCard (pending tasks)
└── Failed Tasks section
    └── Grid of TaskCard (failed tasks with retry)
```

## 3. Component Breakdown

### 3.1 Stats Row

**Container**: `display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px;`
**Responsive**: `< 768px` → `repeat(2, 1fr)`, `< 480px` → `1fr`.

Each stat card is a **KpiCard** with a progress bar underneath the value.

| Stat | Icon | Color | Progress Bar |
|------|------|-------|--------------|
| Active Tasks | `ElIconLoading` | `var(--brand)` | Percent of max concurrent |
| Queued Tasks | `ElIconCollection` | `var(--info)` | Queue depth / capacity |
| Failed Tasks | `ElIconWarning` | `var(--error)` | Failed / total last 24h |
| Queue Depth | `ElIconDataLine` | `var(--warning)` | Total queued + active |

**Progress bar style:**
- Use `ElProgress` with `stroke-width: 8`.
- Custom colors: `color` prop mapped to the stat color above.
- No percentage text inside bar; show raw count as the value.
- Background: `var(--bg-surface)`.

**Data interface**:
```typescript
interface QueueStats {
  active: number;
  queued: number;
  failed: number;
  totalDepth: number;
  maxConcurrent: number;
  queueCapacity: number;
  failedLast24h: number;
  totalLast24h: number;
}
```

### 3.2 TaskCard

**Dimensions**: `min-width: 280px; max-width: 360px;` in a CSS grid (`grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 16px;`).

**Structure** (within `CardPanel`):
```
┌────────────────────────────────────┐
│ [StatusDot] MR Title               │  → title row
│ project / repo                     │  → subtitle (12px, secondary)
│                                    │
│ ████████████░░░░░░  75%            │  → progress bar (ElProgress)
│ Expert: Security  ·  2m 30s        │  → meta row (13px, secondary)
│                                    │
│ [Cancel] [View Logs]               │  → action buttons
└────────────────────────────────────┘
```

**Fields:**

| Element | Style |
|---------|-------|
| Status dot | `StatusBadge` dot-only, 8px, color based on status |
| MR Title | `font-size: 14px; font-weight: 500; color: var(--text-primary);` truncated with ellipsis |
| Project/repo | `font-size: 12px; color: var(--text-secondary);` |
| Progress bar | `ElProgress` `:percentage="task.progress"` with status color |
| Meta row | `font-family: JetBrains Mono; font-size: 12px; color: var(--text-secondary);` |
| Actions | `ElButtonGroup` size="small" |

**Status colors for progress bar:**
- `running` → `var(--brand)` (indigo)
- `queued` → `var(--info)` (blue)
- `failed` → `var(--error)` (red)
- `completed` → `var(--success)` (green) — though completed cards auto-hide after 5 seconds

**Action buttons by status:**

| Status | Primary Action | Secondary Action |
|--------|---------------|----------------|
| running | Cancel (icon: Close) | View Logs (icon: List) |
| queued | Cancel (icon: Close) | — |
| failed | Retry (icon: Refresh) | View Logs (icon: List) |
| completed | — | View Logs (icon: List) |

**Cancel confirmation**: `ElMessageBox.confirm`: "Cancel review for "{mrTitle}"? This action cannot be undone."`

**Task data interface**:
```typescript
interface QueueTask {
  id: string;
  mrTitle: string;
  project: string;
  repository: string;
  status: 'running' | 'queued' | 'failed' | 'completed';
  progress: number; // 0–100
  expertName: string;
  elapsedMs: number;
  createdAt: string;
  startedAt?: string;
  errorMessage?: string;
}
```

### 3.3 Queue Control Bar

**Position**: Between stats row and task grid, or inside PageHeader right side.

**Controls:**
- **Pause / Resume Queue**: `ElButton` with type toggling between "warning" (pause) and "success" (resume).
  - Icon: `ElIconVideoPause` (pause) / `ElIconVideoPlay` (resume).
  - Text: "Pause Queue" / "Resume Queue".
  - On click: `POST /api/v1/queue/pause` or `/api/v1/queue/resume`.
  - On success: `ElNotification` "Queue paused" / "Queue resumed".
  - On error: `ElNotification` error.
  - When paused, all queued task cards show a "⏸ Paused" overlay badge.

- **Cancel All Failed**: `ElButton` type="danger" icon `ElIconDelete`.
  - Confirmation: `ElMessageBox.confirm`: "Cancel all {N} failed tasks?".
  - On confirm: `POST /api/v1/queue/cancel-failed`.
  - On success: failed task cards removed with fade-out animation.

- **Refresh**: `ElButton` icon `ElIconRefresh`. Re-fetches full queue state.

### 3.4 Section Headers

Each section (Active / Queued / Failed) has a sticky header:
```
Active Tasks (3)          ──────────────────────────
```
- Left: section name + count badge (`ElBadge` with count).
- Right: collapse/expand toggle (optional).
- Border-bottom: `1px solid var(--border-color)`.
- Margin-top: `24px` for sections after the first.

## 4. Interactions & State Changes

### 4.1 SSE Updates

- Listen to `queue.update` events.
- Event payload: partial `QueueTask` with updated fields (progress, status, elapsedMs).
- Merge update into existing task by ID.
- If status changes (e.g., running → failed), move task card to appropriate section with CSS transition.
- If new task added, append to grid with fade-in animation (`opacity: 0 → 1`, `translateY: 12px → 0`, `0.25s ease`).
- If task removed, fade-out (`opacity: 1 → 0`, `scale: 1 → 0.95`, `0.2s ease`) then remove from DOM.

### 4.2 Task Progress

- Progress bar updates smoothly via CSS `transition: width 0.3s ease` on the `ElProgress` component.
- Elapsed time updates every second via `setInterval` for running tasks.

### 4.3 Queue Pause State

- When paused:
  - All running tasks continue (they are in-flight), but no new tasks are dequeued.
  - Queued cards show a semi-transparent overlay: `background: rgba(0,0,0,0.3);` with a centered "⏸ Paused" text.
  - Stats row "Queue Depth" turns amber to indicate paused state.

### 4.4 Empty States

- **Active Tasks empty**: Centered text "No active tasks" + `ElIconCheck` in green.
- **Queued Tasks empty**: Centered text "Queue is empty" + `ElIconInfoFilled` in blue.
- **Failed Tasks empty**: Centered text "No failed tasks" + `ElIconCheck` in green.
- Hide empty sections entirely (no section header) unless the section was previously populated (to avoid layout shift).

## 5. Responsive Behavior

| Breakpoint | Task Grid |
|------------|-----------|
| ≥1280px | 4 columns |
| 1024–1279px | 3 columns |
| 768–1023px | 2 columns |
| <768px | 1 column |

Stats row: 4 columns → 2 columns on mobile.

## 6. Animation Details

- Page enter: `page-enter` transition.
- Task card enter: `opacity: 0 → 1`, `translateY: 12px → 0`, `0.25s cubic-bezier(0.4, 0, 0.2, 1)`.
- Task card leave: `opacity: 1 → 0`, `scale: 1 → 0.95`, `0.2s ease`.
- Task card move between sections: `transition: all 0.3s ease` on the card wrapper (using Vue `<TransitionGroup>` with `move-class`).
- SSE progress update: `transition: width 0.3s ease` on progress bar.
- Pause overlay: `transition: opacity 0.2s ease`.
- Stats number changes: `transition: color 0.2s ease` (flash amber when increasing).

## 7. Data Structures

```typescript
// Pinia store: queue.ts
interface QueueState {
  tasks: QueueTask[];
  stats: QueueStats | null;
  isPaused: boolean;
  loading: boolean;
  sseConnected: boolean;
}

// API endpoints
GET  /api/v1/queue               → { tasks: QueueTask[]; stats: QueueStats; isPaused: boolean }
POST /api/v1/queue/pause         → { success: boolean; isPaused: boolean }
POST /api/v1/queue/resume        → { success: boolean; isPaused: boolean }
POST /api/v1/queue/{id}/cancel   → { success: boolean }
POST /api/v1/queue/{id}/retry    → { success: boolean }
POST /api/v1/queue/cancel-failed → { success: boolean; cancelled: number }

// SSE event: queue.update
interface QueueUpdateEvent {
  taskId: string;
  updates: Partial<QueueTask>;
}
```
