# Blueprint split-sync-worker

## Feature description

Move the todo sync loop out of the API process into a separate worker, so slow sync passes no longer block API requests.

Today the API process runs the sync loop on a timer when `SYNC_IN_PROCESS=true`. Local `server/.env` sets it, so local runs sync inside the API and the worker binary is unused. After this change:

- The worker owns the scheduled sync loop.
- The API only enqueues sync requests and never runs a pass itself.
- `SYNC_IN_PROCESS` is removed; `SYNC_WORKER_CONCURRENCY` controls the worker.

## Decisions

- **One worker, one queue.** The worker reads one `sync` queue. No per-list queues.
- **One parser for worker env.** `SYNC_WORKER_CONCURRENCY` and `SYNC_BATCH_SIZE` go through the same trim/parse/positive-int rules. Invalid values throw at startup.
- **API enqueues, never runs.** `TodoService` calls `syncQueue.enqueue(listId)` after a write. It does not import `runSyncPass()`.
- **Dedupe by list id.** Enqueueing a list that is already queued is a no-op. A burst of edits => one sync pass.
- **Warn, do not fail, on leftover `SYNC_IN_PROCESS`.** Startup logs that the var is ignored.
- **`.env.example` documents both vars.** Concurrency defaults to 2, batch size to 100.

## Flow

- Process start -> parse worker env (concurrency, batch size).
- Todo write -> `TodoService` -> `syncQueue.enqueue(listId)`.
- Worker -> dequeue list id -> `runSyncPass(listId)` -> ack.
- Pass fails -> retry with backoff (max 3) -> dead-letter.
- Invalid env -> throw before the worker starts.

## Contracts

- With both vars unset the worker runs 2 passes at a time with batch size 100.
- The API process never calls `runSyncPass()`.
- Enqueueing the same list twice before a pass starts yields one pass.
- A failed pass retries at most 3 times, then lands in the dead-letter queue.
- Unsupported non-empty values for either var fail at startup.
- `SYNC_IN_PROCESS` has no effect and logs a warning when set.
- Existing sync results stay the same: same items pushed, same conflict rules.

## Out of scope

- Per-list sync intervals.
- Changing the conflict resolution rules.
- Deploy workflow env injection.
- Autoscaling the worker.
- The legacy `POST /sync/push` handler.
- Moving `scripts/backfill.ts` onto the queue.

## Gotchas

- **Leftover `SYNC_IN_PROCESS=true` in a gitignored `.env` does nothing now.** Startup must warn so nobody waits for in-process sync.
- **Dedupe key is the list id, not the item id.** Keying by item would run one pass per edited item.
- **`SETTINGS` is import-time.** Parser tests that mutate `process.env` after first import will not see `SETTINGS` change; test the parser directly.
- **Worker and API share the database pool config.** Raising concurrency without raising the pool size starves the API.

## Open questions

- None. Worker env names are `SYNC_WORKER_CONCURRENCY` and `SYNC_BATCH_SIZE`.
