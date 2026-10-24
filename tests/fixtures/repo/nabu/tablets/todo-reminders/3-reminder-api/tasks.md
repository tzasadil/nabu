# TODO Checklist

## Anchors

- `server/src/reminder-api/` — module (service, controller, DTOs)
- `server/src/reminder-api/reminder-api.service.ts` — create, snooze, cancel
- `server/src/reminder-api/reminder.controller.ts` — POST + GET + PATCH + DELETE
- `server/src/reminder-api/dto/` — Zod schemas + response DTOs
- `server/src/todos/todo.service.ts` — cascade on delete
- `server/src/reminders/reminder-schedule.service.ts` — schedule dependency (slice 1)
- `server/src/reminder-delivery/reminder-delivery.service.ts` — `sendNow` (slice 2)

## 1. Pre-implementation gate

- [x] 1.1 Read `specs/reminder-api/spec.md` and `blueprint.md`; confirm slice 3 only (API + snooze + cancel). No new scanner or delivery code.
- [x] 1.2 Confirm slice 1 contract: `ReminderScheduleService.create`, `findDue`.
- [x] 1.3 Confirm slice 2 contract: `ReminderDeliveryService.sendNow`.
- [x] 1.4 Confirm auth: list owner or editor for writes, viewer for GET.

## 2. Implement + verify

### 2.1 Schema and types

- [x] 2.1.1 Add `cancelled` to the reminder state enum; generate migration.
- [x] 2.1.2 Define Zod schemas + DTOs: `CreateReminderInput`, `SnoozeReminderInput`, `ReminderResponse`.

### 2.2 Tests first (red)

- [x] 2.2.1 Composite: POST with a future time → 201 and a pending reminder.
- [x] 2.2.2 Composite: POST with a past time → 400.
- [x] 2.2.3 Composite: POST with an unknown time zone → 400.
- [x] 2.2.4 Composite: POST with `sendTest` → one test send, state stays `pending`.
- [x] 2.2.5 Composite: PATCH snooze → new `fireAt`.
- [x] 2.2.6 Composite: PATCH on a claimed reminder → 409.
- [x] 2.2.7 Composite: DELETE → `cancelled`, GET no longer lists it.
- [x] 2.2.8 Composite: GET lists pending reminders ordered by `fireAt`.
- [x] 2.2.9 Composite: viewer POST → 403; viewer GET → 200.
- [x] 2.2.10 Composite: deleting the todo cancels its pending reminders.
- [x] 2.2.11 Controller: no list access → 404.
- [x] 2.2.12 Run tests red before implementation.

### 2.3 Input parsing

- [x] 2.3.1 Implement local time + zone parsing with the slice 1 helper.
- [x] 2.3.2 Pure validator: reject past times after UTC conversion.

### 2.4 API service

- [x] 2.4.1 Implement `ReminderApiService.create`, `snooze`, `cancel`, `listPending`.
- [x] 2.4.2 Wire the optional test send.
- [x] 2.4.3 Cascade cancel inside the todo delete transaction.

### 2.5 HTTP API

- [x] 2.5.1 `ReminderController` POST + GET + PATCH + DELETE.
- [x] 2.5.2 Register `ReminderApiModule` in the app module.

### 2.6 Validation

- [x] 2.6.1 `pnpm --filter server exec vitest src/reminder-api` passes.
- [x] 2.6.2 `pnpm --filter server build` passes.
- [x] 2.6.3 Doc comments on all new exports in the `reminder-api` module (per AGENTS.md).

## 3. Completion gate

- [x] 3.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 3.2 Reread spec; update contracts only if implementation diverged (no implementation details in spec).
- [x] 3.3 Update `backlog.md` with deferred items; remove resolved.
