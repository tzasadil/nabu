# Blueprint 42-empty-list-notice

## Feature description

The list page renders blank when a list has no items, so users think the app is broken. Issue 42 shows a blank page for a new list with no todos. Show a short `No todos` notice when a list has no items, and keep the normal view for every other case.

## Decisions

- **Fix in `ListPage` only.** That component renders every list view: own lists, shared lists, and the today view. One branch there covers every caller. Do not add per-view checks.
- **Empty means zero items after load.** `items.length === 0` after a successful fetch renders the notice. A fetch still in progress keeps the spinner.
- **Keep load errors as they are.** A failed fetch is a different failure. Leave that path alone.
- **Reuse the shared `Notice` component.** No new styles. The notice text is the bare label `No todos`.
- **Do not change the API.** The server already returns `items: []` for an empty list. The bug is client-side only.

## Flow

- Open list -> `ListPage` -> fetch items
- Loading -> spinner
- Loaded, zero items -> `EmptyListNotice`
- Loaded, one or more items -> normal list view
- Fetch failed -> unchanged

## Contracts

- A list with zero items MUST show the `No todos` notice instead of a blank page.
- A list with items MUST render as before.
- The notice MUST NOT show while the fetch is in progress.
- A failed fetch MUST behave as before.
- No new API field, route, or error code.

## Out of scope

- Sidebar counts for empty lists.
- An "Add todo" button inside the notice.
- A separate message for filtered-to-empty.
- Error handling for failed fetches.
- Server-side changes.

## Gotchas

- `items` is `undefined` until the first fetch resolves. Check loading state first, or the notice flashes on every page open.
- Shared lists reuse `ListPage` with a read-only flag. The notice must render there too.
- The today view filters by due date on the client, so an empty today view hits the filtered case, not the empty case.
- Snapshot tests for `ListPage` only cover a non-empty list. Add the empty case there.
- Screen readers need the notice in a live region, or the change from spinner to notice is silent.

## Open questions

- None. Empty shows `No todos`; everything else stays the same.
