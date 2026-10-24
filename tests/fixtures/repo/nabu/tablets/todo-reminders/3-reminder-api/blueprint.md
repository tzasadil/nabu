# Blueprint todo-reminders-3

## Decisions

- **Module `reminder-api`** at `server/src/reminder-api/` — controller for `POST`, `GET`, `PATCH`, `DELETE` under `/todos/:todoId/reminders`; imports `RemindersModule`, `ReminderDeliveryModule`, `TodosModule`.
- **Create flow** — `ReminderApiService.create(todoId, input)`:
  1. Check todo access (auth done in controller first).
  2. Parse local time + zone; reject times in the past.
  3. `ReminderScheduleService.create` from slice 1.
  4. If `sendTest` is set, call `ReminderDeliveryService.sendNow` from slice 2.
  5. Map to response DTO; return.
- **Input shape** — `{ at: "2026-10-24T09:00", timeZone: "Europe/Prague", sendTest?: boolean }`. The client sends local time; the server converts.
- **Snooze is PATCH** — `PATCH /todos/:todoId/reminders/:reminderId` with a new `at`. Only `pending` reminders can be snoozed.
- **Auth** — Owner or editor of the list may write; viewers may only GET (404 if no access).
- **No admin routes here** — scanner admin stays in slice 1.
- **Cancel is DELETE** — Sets `state = cancelled`; the row stays for history.
- **Pending only on GET** — GET lists `pending` reminders ordered by `fireAt`.
- **Validation** — Zod at the boundary; unknown time zone → 400.
- **Todo deleted** — All its pending reminders are cancelled in the same transaction.
- **Rate model** — No per-user limit in v1 (backlog).
- **Response times** — `fireAt` in UTC only in v1.

## Flow

1. `POST /todos/:todoId/reminders` → auth
2. Parse and validate input
3. If time in the past → 400
4. Create the reminder (slice 1); optional test send (slice 2)
5. Build `ReminderResponseDto` → return 201
6. `GET` → list pending reminders for the todo or 404

## Boundaries

- **In scope:** Reminder API service, controller, DTOs/Zod, snooze, cancel, cascade on todo delete, composite tests (create, past time, bad zone, snooze, cancel, viewer access, test send).
- **Out of scope:** Reminder history UI, recurring reminders, snooze presets, bulk endpoints, local time on the response.
- **Depends on:** `ReminderScheduleService.create`, `ReminderScheduleService.findDue`, `ReminderDeliveryService.sendNow`.

## Gotchas

- **Snooze a claimed reminder** — A reminder the scanner already claimed cannot be snoozed; return 409.
- **Past time check** — Compare in UTC after conversion, not on the raw local string.
- **Shared lists** — Editors can add reminders, but delivery goes to the reminder's creator only.
- **Test send** — `sendTest` must not change the reminder state.
- **Cascade** — Cancel reminders in the same transaction as the todo delete, or the scanner may fire one in between.
- **Large reminder lists** — Many reminders ⇒ long GET; acceptable v1; cap deferred to backlog.
