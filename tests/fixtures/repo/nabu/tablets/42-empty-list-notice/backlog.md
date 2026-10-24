<!---
Deferred scope items for this change. OOS features, possible risks that may turn out fine, major refactors, etc.
-->

### Archived empty lists still show in the sidebar

- **What:** Archived lists with zero items still appear in the sidebar count. After the fix the list page shows a notice, but the sidebar still counts them.
- **Where:** `web/src/sidebar/ListNav.tsx`, `server/src/lists/list-summary.service.ts`
- **Why deferred:** The issue is about the list page. Changing sidebar counts is a separate product call.
- **Possible resolutions:** Hide archived empty lists, or count only lists with open items.
- **When to revisit:** Users report the sidebar count does not match what they see.

### Notice has no call to action

- **What:** The notice only says `No todos`. It does not offer an "Add todo" button.
- **Where:** `web/src/lists/EmptyListNotice.tsx`
- **Why deferred:** Issue asks for a notice, not a new flow. A button needs design input.
- **Possible resolutions:** Reuse the existing `AddTodoButton` inside the notice.
- **When to revisit:** Usage data shows people leave empty lists without adding items.

### Filtered-to-empty uses the same notice

- **What:** When a filter hides every item, the page shows the same `No todos` notice instead of `No matches`.
- **Where:** `web/src/lists/ListPage.tsx`, `web/src/lists/filters.ts`
- **Why deferred:** The reported bug is the truly empty list. A second message needs copy.
- **Possible resolutions:** Pass the filter state to the notice and pick the label from it.
- **When to revisit:** Users read the notice as data loss after filtering.

### Load errors still blank the page

- **What:** A failed list fetch still renders an empty page with no message.
- **Where:** `web/src/lists/ListPage.tsx`
- **Why deferred:** Different failure from an empty list; not in the issue.
- **Possible resolutions:** Show the shared `ErrorNotice` on fetch failure.
- **When to revisit:** A load failure is reported as "my list is empty".
