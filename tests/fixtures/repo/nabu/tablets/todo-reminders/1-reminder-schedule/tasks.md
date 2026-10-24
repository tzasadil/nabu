# TODO Checklist

## Anchors

- `server/src/reminders/` — module (schedule service, scanner, sink, controller)
- `server/src/reminders/reminder-schedule.service.ts` — create, cancel, find due
- `server/src/reminders/reminder-scanner.ts` — tick loop + claim
- `server/src/reminders/reminder-sink.ts` — hand-off interface + logging sink
- `server/src/reminders/reminder-admin.controller.ts` — `POST /admin/reminders/scan`, `GET /admin/reminders/status`
- `server/.env.example` — `REMINDER_SCAN_INTERVAL_MS`, `REMINDER_LOOKAHEAD_MS`

## 1. Pre-implementation gate

- [x] 1.1 Read `specs/reminder-schedule/spec.md` and `blueprint.md`; confirm slice 1 scope only (table + scanner + status). No delivery or user API.
- [x] 1.2 Confirm the scanner and API share one database.
- [x] 1.3 Confirm the claim statement works on the deployed database version.
- [x] 1.4 Confirm admin auth pattern for `/admin/reminders/*` (reuse the existing admin guard).

## 2. Implement + verify

### 2.1 Types and env first

- [x] 2.1.1 Add `REMINDER_SCAN_INTERVAL_MS` and `REMINDER_LOOKAHEAD_MS` to `server/.env.example` and env validation.
- [x] 2.1.2 Define `Reminder` (`id`, `todoId`, `fireAt`, `timeZone`, `state`) and `ReminderScanStatus`.

### 2.2 Tests first (red)

- [x] 2.2.1 Unit test: local time + zone converts to UTC `fireAt`; DST gap maps to the next valid minute.
- [x] 2.2.2 Composite test: two due reminders → both claimed, both handed to the sink, status reports `success`.
- [x] 2.2.3 Composite test: two scanners on one tick never claim the same row.
- [x] 2.2.4 Composite test: sink failure puts the row back to `pending`.
- [x] 2.2.5 Composite test: a tick with no due rows reports `success` and changes nothing.
- [x] 2.2.6 Composite test: startup does not wait for the first tick.
- [x] 2.2.7 Controller test: `GET /admin/reminders/status` returns enabled flag and last scan fields; `POST /admin/reminders/scan` runs a scan; non-admin gets 403.
- [x] 2.2.8 Run new tests and confirm they fail before production implementation.

### 2.3 Reminder table

- [x] 2.3.1 Add the `reminders` migration with an index on `(state, fireAt)`.
- [x] 2.3.2 Implement `ReminderScheduleService`: create, cancel, `findDue` (internal, for slice 3).

### 2.4 Scanner

- [x] 2.4.1 Implement the claim statement.
- [x] 2.4.2 Implement `ReminderScanner` tick: claim → sink → mark sent or pending → update status.
- [x] 2.4.3 Start the scanner after listen when enabled; disabled → `enabled: false`.

### 2.5 Admin API

- [x] 2.5.1 Register `RemindersModule` in the app module.
- [x] 2.5.2 Implement admin controller endpoints per spec.

### 2.6 Validation

- [x] 2.6.1 `pnpm --filter server test` passes (new + existing).
- [x] 2.6.2 `pnpm --filter server build` passes for touched files.
- [x] 2.6.3 Mandatory composite tests from 2.2 pass locally.

## 3. Completion gate

- [x] 3.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 3.2 Reread delta specs; fold implementation insights into specs (contracts only, no code paths).
- [x] 3.3 Review `backlog.md`; drop resolved items, keep deferred.
