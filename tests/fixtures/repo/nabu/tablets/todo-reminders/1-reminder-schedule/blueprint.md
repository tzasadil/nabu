# Blueprint todo-reminders-1

## Decisions

- **Own table, not a todo column** — Reminders live in a `reminders` table with `todoId`, `fireAt` (UTC), `timeZone`, `state`. A todo can have more than one reminder, so a column on `todos` does not fit.
- **Store UTC plus the zone** — `fireAt` is stored in UTC. `timeZone` keeps the user's zone so the API can show local time. Convert at the boundary only.
- **Scan, do not push** — A `ReminderScanner` runs every `REMINDER_SCAN_INTERVAL_MS` (default 30 s) and selects rows with `fireAt <= now + lookahead` and `state = pending`.
- **Claim before hand-off** — The scanner flips due rows to `claimed` in the same statement that selects them. Two scanners never claim the same row.
- **Hand-off behind an interface** — `ReminderSink` with `deliver(reminder)`. Slice 1 ships a logging sink; slice 2 replaces it.
- **Startup is fire-and-forget** — `ReminderScanner` starts after the server listens; a failing tick logs and retries on the next tick, never blocks startup.
- **Status storage** — In-memory status is enough for v1 (`pendingCount`, `lastScanAt`, `lastScanOutcome`, `lastScanError`). No history table in this slice.
- **Admin endpoints** — Module `reminders` under `server/src/reminders/`:
  - `POST /admin/reminders/scan` — run one scan now (admin guard)
  - `GET /admin/reminders/status` — scanner status
  The user-facing reminder API is **out of scope** for this slice (slice 3 adds it); the internal `findDue` method is still implemented for slice 3 reuse.
- **Unconfigured = off** — `REMINDER_SCAN_INTERVAL_MS=0` disables the scanner and reports `enabled: false`.

## Flow

1. Server starts → migration creates `reminders` if missing
2. If scanning is enabled → start `ReminderScanner`
3. Tick → claim due rows → hand each to `ReminderSink` → mark `sent` or back to `pending`
4. Status updated (`pendingCount`, `lastScanAt`, outcome)
5. Operator may call `POST /admin/reminders/scan` anytime

## Boundaries

- **In scope:** `reminders` table, `ReminderScheduleService`, `ReminderScanner`, logging `ReminderSink`, env vars, admin status + scan endpoints, composite tests with a fake clock.
- **Out of scope:** Push and email delivery (`todo-reminders-2`), user-facing API (`todo-reminders-3`), scan history, recurring reminders, UI.
- **Assumption:** One database for API and scanner. Confirm at the pre-implementation gate.

## Gotchas

- **Do not reuse the sync scheduler** — `SyncScheduler` runs per list; reminders need one global tick.
- **Clock drift** — Compare against database `now()`, not the server clock, or replicas disagree on what is due.
- **Empty scan** — A tick with no due rows is a success with `pendingCount` unchanged, not an error.
- **Sink failure** — If the sink throws, put the row back to `pending` so the next tick retries it.
- **Multi-instance deployments** — Each replica runs a scanner; the claim step keeps them from sending twice. `GET /status` is per process.
- **DST gaps** — A local time that does not exist (spring forward) maps to the next valid minute.

## Open questions

- Exact lookahead window (v1 uses 0; raise if delivery latency matters).
