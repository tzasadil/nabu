<!---
Deferred scope items for this change. OOS features, possible risks that may turn out fine, major refactors, etc.
-->

### Deploy / CI env injection

- **What:** Local gitignored `server/.env` no longer sets `SYNC_IN_PROCESS=true` (commented out in the workspace and main checkout). `deploy/env/*.env.example` and deploy workflows still do not mention either sync var. Hosted secret stores were not changed.
- **Where:** `deploy/env/`, `.github/workflows/deploy-*.yml`, hosted secret stores.
- **Why deferred:** Those files never documented the existing var; this change does not add a deploy matrix.
- **Possible resolutions:** Add commented lines to deploy env examples; unset `SYNC_IN_PROCESS` in any hosted secret store that still has `true`.
- **When to revisit:** First environment that still runs sync inside the API after this ships, or an ops request to template the new var.

### SyncClient still reads `process.env` directly

- **What:** `SyncClient` builds its HTTP agent from `process.env` rather than `SETTINGS`.
- **Where:** `server/src/sync/sync-client.ts`
- **Why deferred:** Pre-existing; folding all of its env reads into `SETTINGS` is unrelated to the worker split.
- **Possible resolutions:** Keep parsing `SYNC_BATCH_SIZE` at construction in this change; later move the class onto `SETTINGS` if a cycle-free import is easy.
- **When to revisit:** Next time this client is touched for config.

### Generated ops docs

- **What:** `docs/ops/sync.md` describes the sync loop but does not list the env vars.
- **Where:** `docs/ops/sync.md`
- **Why deferred:** The ops docs are generated from source; do not hand-edit generated pages unless asked.
- **When to revisit:** After the next docs refresh, if the env split is still missing.

### Legacy push endpoint stays in the API

- **What:** The old `POST /sync/push` handler still runs a sync pass inline. It reads `SYNC_IN_PROCESS` (default off), not the worker queue.
- **Where:** `server/src/sync/sync.controller.ts`
- **Why deferred:** User asked for the scheduled loop only. Moving the push handler onto the queue changes its response contract.
- **Possible resolutions:** Delete the endpoint if no client still calls it; or enqueue and return `202`.
- **When to revisit:** Push traffic shows up in API latency after this ships.

### Close the direct `runSyncPass()` import

- **What:** `scripts/backfill.ts` imports `runSyncPass()` directly instead of enqueueing work. This change leaves that one call site as is.
- **Where:** `server/scripts/backfill.ts`, `server/src/sync/sync-pass.ts`
- **Why deferred:** Routing the backfill through the queue changes its retry and timeout behaviour.
- **Possible resolutions:** Enqueue backfill batches; stop exporting `runSyncPass()` outside the worker.
- **When to revisit:** Next new direct caller of `runSyncPass()`.

### Per-list sync interval removed

- **What:** `SyncOptions.intervalMs` remains for explicit overrides. Shared lists no longer read a list-specific interval env.
- **Where:** `SyncOptions.intervalMs`
- **Why deferred:** No remaining caller needs a list-specific interval. The unused `parseListSyncInterval` helper was deleted.
- **Possible resolutions:** Bring back a named env only if one list type must sync more often.
- **When to revisit:** Product asks for faster sync on shared lists only.
