<!---
Deferred findings for this change: **not** part of the delta spec contract (`specs/**/*.md`).

**Triage:** Record deferred items here until the change is archived.
-->

### Recurring reminders

- **What:** Repeat rules (daily, weekly) instead of one fire time per reminder.
- **Where:** `ReminderScheduleService`, `next-fire.ts`.
- **Why deferred:** One-shot reminders cover v1; add when users ask for repeats.

### Scanner pagination

- **What:** Page through due reminders instead of loading every due row in one query.
- **Where:** `ReminderScanner`.
- **Why deferred:** Due rows per tick stay small until real usage data says otherwise.

### Persist scan history

- **What:** Store scan runs (startedAt, duration, count, error) for audit.
- **Where:** New table + `ReminderScanner`.
- **Why deferred:** In-memory status covers slice 1; revisit if operators need history.

### Public due-reminders endpoint

- **What:** `GET /reminders/due` for debugging or other clients.
- **Where:** `reminders` controller.
- **Why deferred:** Slice 3 owns the reminder API; the internal `findDue` method is enough for now.

### Per-user time zone

- **What:** Store a time zone per user instead of one per reminder.
- **Where:** `ReminderScheduleService` normalization.
- **Why deferred:** v1 stores the time zone on each reminder. **Risk:** a user who moves keeps old time zones on old reminders; slice 3 should show the stored zone next to the time.

### Fixture reminders for local development

- **What:** `REMINDER_FIXTURE_PATH` to seed reminders locally.
- **Where:** `ReminderScheduleService` seed hook.
- **Why deferred:** Enables slice 2–3 demos without real data; add when the API slice starts.

### Shared scheduler loop

- **What:** Merge `ReminderScanner` and `SyncScheduler` behind one tick loop.
- **Where:** `server/src/shared/scheduler/` or similar.
- **Why deferred:** Slice 1 ships a focused module; refactor when a third loop appears.

### Centralize reminder env in SETTINGS

- **What:** Move `REMINDER_SCAN_INTERVAL_MS` / `REMINDER_LOOKAHEAD_MS` into `settings.ts` like `SYNC_BATCH_SIZE`.
- **Where:** `server/src/config/settings.ts`, `reminder-env.ts`.
- **Why deferred:** Slice 1 uses direct env reads; low risk for v1.

### Extract shared time helpers

- **What:** Share `toUtcInstant` with the due-date code in `todos/`.
- **Where:** `server/src/shared/time/instant.ts`.
- **Why deferred:** Same logic; refactor when the due-date code changes next.

### Eval: polling vs delayed queue

- **What:** Compare a polling scanner with delayed queue messages when reminder volume stays small (&lt;500 per minute).
- **Where:** slice 2 design spike.
- **Why deferred:** User chose polling in outline; measure before switching.
