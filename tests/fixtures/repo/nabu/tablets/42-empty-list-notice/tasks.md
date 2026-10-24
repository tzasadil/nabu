# TODO Checklist

## Anchors

- `web/src/lists/ListPage.tsx`
- `web/src/lists/__tests__/ListPage.test.tsx`

## 1. Implement + verify

- [x] 1.1 In `ListPage`, render `EmptyListNotice` when the fetch succeeded and `items.length === 0`. Keep the spinner while loading.
- [x] 1.2 Build `EmptyListNotice` on the shared `Notice` component with the label `No todos` inside a live region.
- [x] 1.3 Extend `ListPage.test.tsx` with AAA coverage for: empty list shows the notice; non-empty list renders items; loading shows no notice.
- [x] 1.4 Capture newly discovered follow-up work in `backlog.md` instead of silently changing scope.
- [x] 1.5 Run focused validation: the `ListPage` test file (Vitest).

## 2. Completion gate

- [x] 2.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 2.2 Fill backlog.md with follow up work. Remove already fixed.
