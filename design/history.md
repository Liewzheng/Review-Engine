# Review History Page Design

## 1. Route & Purpose

- **Route**: `/history`
- **Purpose**: Browse, search, and filter all code review records. The primary destination for investigating past reviews.
- **Data**: Fetched on mount + when filters/search/pagination change. No SSE (static after fetch).

## 2. Page Layout

```
PageHeader
├── Title: "Review History"
└── Right: Export button (ElButton icon: Download) — optional future feature

Filter Bar (margin-bottom: 16px)
├── Search input (ElInput, prefix: Search icon, placeholder: "Search MR title, author, branch...")
├── Project filter (ElSelect, placeholder: "All Projects")
├── Status filter (ElSelect, placeholder: "All Statuses")
├── Date range picker (ElDatePicker, type: daterange)
├── Repository filter (ElSelect, placeholder: "All Repositories")
└── Reset filters button (ElButton, text: "Reset", icon: Close)

Data Table (full width)
└── DataTable with pagination footer

Pagination Footer
└── ElPagination + page size selector (25 / 50 / 100)
```

## 3. Component Breakdown

### 3.1 Filter Bar

**Container**: `display: flex; flex-wrap: wrap; gap: 12px; align-items: center; padding: 16px 20px; background: var(--bg-card); border: 1px solid var(--border-color); border-radius: var(--radius-md);`

**Fields:**

| Field | Component | Width | Options / Placeholder |
|-------|-----------|-------|-----------------------|
| Search | `ElInput` | `min-width: 280px; flex: 1;` | placeholder: "Search MR title, author, branch..." |
| Project | `ElSelect` | `160px` | Options: fetched from `/api/v1/projects` |
| Status | `ElSelect` | `140px` | Options: All, Queued, Running, Completed, Failed, Cancelled |
| Date Range | `ElDatePicker` | `260px` | type: daterange, format: YYYY-MM-DD |
| Repository | `ElSelect` | `180px` | Options: fetched from `/api/v1/repositories` |
| Reset | `ElButton` | auto | icon: `ElIconClose`, text: "Reset" |

**Filter behavior:**
- All filters are applied client-side if total dataset < 1000, otherwise server-side via query params.
- Debounce search input: `300ms` after user stops typing.
- Changing any filter resets pagination to page 1.
- Reset button clears all filters and reloads default state.

**Filter state in URL:**
- On filter change, update query params: `?project=foo&status=failed&from=2024-01-01&to=2024-01-31&repo=bar&q=search-term&page=1&size=25`
- On page load, read query params and populate filters.
- Use Vue Router `replace` to avoid polluting history stack.

### 3.2 Data Table

**Container**: `DataTable` with `height: calc(100vh - 280px)` for sticky header on tall pages.

**Columns:**

| Column | Width | Content | Sortable |
|--------|-------|---------|----------|
| MR Title | flexible | Title + project tag + branch name (caption) | yes |
| Project | 140px | `ElTag` with project name | yes |
| Author | 140px | Avatar (28px) + name | yes |
| Status | 110px | `StatusBadge` | yes |
| Duration | 100px | `JetBrains Mono` formatted duration | yes |
| Created | 150px | Relative time + tooltip with absolute | yes |
| Actions | 120px | Button group | no |

**Row details:**

**MR Title cell**:
```
[Project Tag]  MR Title Text
               branch: feature/login-page → main
```
- Title: `font-size: 13px; color: var(--text-primary); font-weight: 500;`
- Branch: `font-size: 11px; color: var(--text-secondary); font-family: JetBrains Mono;`
- Project tag: `ElTag` size="small", type="info".

**Author cell**:
- Avatar: `width: 28px; height: 28px; border-radius: 50%;` with fallback to initials if no image.
- Name: `margin-left: 8px; font-size: 13px;`

**Status cell**: `StatusBadge` component.
- `queued` → blue "Queued"
- `running` → green spinner "In Progress"
- `completed` → green "Completed"
- `failed` → red "Failed"
- `cancelled` → gray "Cancelled"

**Actions cell** (button group, `ElButtonGroup`):
- **Re-review**: `ElButton` size="small" icon `ElIconRefresh`. Tooltip: "Re-run review".
- **Details**: `ElButton` size="small" icon `ElIconArrowRight`. Tooltip: "View details".
- **More**: `ElButton` size="small" icon `ElIconMore`. Dropdown menu:
  - "View original comment" → opens external link to GitLab/GitHub MR comment
  - "Copy review ID" → copies to clipboard, shows toast
  - "View logs" → navigates to `/logs?reviewId={id}`

**Row hover**: `background: var(--bg-hover); cursor: pointer;`. Clicking the row (not buttons) opens the detail drawer.

### 3.3 Detail Drawer

**Component**: `ElDrawer` with `size: 600px` (50% on mobile, full width on <768px).

**Drawer sections:**
1. Header: MR Title + `ElTag` for status + close button.
2. Meta grid (2 columns): Author, Project, Branch, Created At, Duration, Commit SHA.
3. Expert results: Accordion (`ElCollapse`) showing each expert's findings.
   - Each item: Expert name + `StatusBadge` (success/warning/error) + score if available.
   - Content: Comment text preview, expandable to full.
4. Raw data: `ElTabs` with tabs "Summary", "Full Comment", "API Response".
   - "Full Comment": raw markdown rendered with `ElInput` type="textarea" readonly.
   - "API Response": JSON pretty-print in `JetBrains Mono` inside a scrollable `<pre>` block.
5. Footer actions: "Re-run Review", "View on GitLab", "Close Drawer".

**Drawer data**:
```typescript
interface ReviewDetail {
  id: string;
  mrTitle: string;
  project: string;
  repository: string;
  branch: string;
  targetBranch: string;
  author: { name: string; avatarUrl?: string; email?: string };
  status: 'queued' | 'running' | 'completed' | 'failed' | 'cancelled';
  durationMs: number;
  createdAt: string;
  completedAt?: string;
  commitSha: string;
  experts: ExpertResult[];
  rawComment?: string;
  rawApiResponse?: object;
  gitlabMrUrl?: string;
}

interface ExpertResult {
  expertId: string;
  expertName: string;
  status: 'success' | 'warning' | 'error' | 'skipped';
  score?: number; // 0-100
  summary: string;
  details?: string;
}
```

### 3.4 Pagination

**Component**: `ElPagination` + `ElSelect` for page size.

**Layout**: `display: flex; justify-content: space-between; align-items: center; padding: 16px 0;`
- Left: "Showing {start} to {end} of {total} reviews" (12px, `--text-secondary`)
- Right: `ElPagination` + `ElSelect` (options: 25, 50, 100)

**Behavior:**
- Page size change resets to page 1.
- Pagination state synced to URL query param `page` and `size`.
- On filter change, reset to page 1.

## 4. Interactions & State Changes

### 4.1 Search

- Input event debounced at `300ms`.
- On debounced input: update URL query param `q`, reset page to 1, fetch new data.
- Empty search string = show all (no `q` param).

### 4.2 Re-review Action

- Click "Re-review" button → `ElMessageBox.confirm` with text: "Re-run review for "{mrTitle}"? This will post a new comment to the MR."`
- On confirm: `POST /api/v1/reviews/{id}/rerun`
- On success: `ElNotification` "Review re-queued" + the row status updates to "Queued".
- On error: `ElNotification` error + row shake animation (optional).

### 4.3 View Original Comment

- Click "View original comment" in dropdown → open `gitlabMrUrl` in new tab (`window.open(url, '_blank')`).
- If URL unavailable, show `ElNotification` "Original comment URL not available" (warning).

## 5. Responsive Behavior

| Breakpoint | Changes |
|------------|---------|
| ≥1280px | Full filter bar inline, table with all columns |
| 1024–1279px | Filter bar wraps, table with all columns |
| 768–1023px | Filter bar stacks, hide "Project" and "Repository" columns, drawer 50% width |
| <768px | Filter bar stacks, only show Title, Status, Actions columns; drawer full width; pagination compact |

## 6. Animation Details

- Page enter: `page-enter` transition (0.2s fade + slide).
- Filter bar: slide down `0.15s` when expanding on mobile.
- Table rows: no entrance animation.
- Drawer: `ElDrawer` default slide animation (0.3s from right).
- Row on re-review: `animation: shake 0.4s ease-in-out` on error (optional).
- Skeleton: `animated: true` on initial load.

## 7. Data Fetching

```typescript
// Pinia store: history.ts
interface HistoryState {
  reviews: ReviewListItem[];
  total: number;
  page: number;
  pageSize: number;
  filters: HistoryFilters;
  loading: boolean;
  selectedReview: ReviewDetail | null;
  drawerOpen: boolean;
}

interface HistoryFilters {
  q: string;
  project: string | null;
  status: string | null;
  dateFrom: string | null;
  dateTo: string | null;
  repository: string | null;
}

interface ReviewListItem {
  id: string;
  mrTitle: string;
  project: string;
  repository: string;
  branch: string;
  targetBranch: string;
  author: { name: string; avatarUrl?: string };
  status: 'queued' | 'running' | 'completed' | 'failed' | 'cancelled';
  durationMs: number;
  createdAt: string;
  gitlabMrUrl?: string;
}

// API endpoints
GET /api/v1/reviews?{filters}&page={page}&size={size}  → { items: ReviewListItem[]; total: number }
GET /api/v1/reviews/{id}                                  → ReviewDetail
POST /api/v1/reviews/{id}/rerun                          → { success: boolean; newReviewId: string }
GET /api/v1/projects                                      → string[]
GET /api/v1/repositories                                  → string[]
```
