# TODO Checklist — split-sync-worker

## Anchors

- `server/src/sync/sync-env.ts`
- `server/src/sync/sync-env.spec.ts`
- `server/src/config/settings.ts`
- `server/src/sync/sync-queue.ts`
- `server/src/sync/sync-pass.ts`
- `server/src/worker/main.ts`
- `server/src/todos/todo.service.ts`
- `server/.env.example`
- `server/tests/sync/sync-worker.spec.ts`

## 1. Parsers + queue

- [x] 1.1 Add one parser for `SYNC_WORKER_CONCURRENCY` and `SYNC_BATCH_SIZE`: trim, positive int, unset -> default, invalid -> throw with the env name.
- [x] 1.2 Add `syncQueue` with `enqueue(listId)` that dedupes by list id.
- [x] 1.3 Expose both parsed values on `SETTINGS`. Parse once.
- [x] 1.4 Parser unit tests: defaults, valid values, invalid throws with the right env name.

## 2. Wire the worker

- [x] 2.1 Worker entry point dequeues list ids and calls `runSyncPass(listId)`.
- [x] 2.2 Retry a failed pass with backoff, max 3, then dead-letter.
- [x] 2.3 `TodoService` enqueues after every write instead of calling `runSyncPass()`.
- [x] 2.4 Remove the in-process timer from the API bootstrap.
- [x] 2.5 Log a warning at startup when `SYNC_IN_PROCESS` is set.
- [x] 2.6 Worker respects `SYNC_WORKER_CONCURRENCY`.
- [x] 2.7 Update `server/.env.example`: document both vars and their defaults; drop `SYNC_IN_PROCESS`.

## 3. Verify (automated)

- [x] 3.1 Queue test: two enqueues of the same list before a pass yield one pass.
- [x] 3.2 Worker test: failed pass retries 3 times, then dead-letters. Mock the sync client.
- [x] 3.3 API test: a todo write enqueues and never calls `runSyncPass()`.
- [x] 3.4 Run focused server tests: `sync-env.spec.ts`, `sync-worker.spec.ts`, `todo.service.spec.ts`. No real network.

## 4. Completion gate

- [x] 4.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 4.2 Fill `backlog.md` with follow-up work. Remove already fixed.
