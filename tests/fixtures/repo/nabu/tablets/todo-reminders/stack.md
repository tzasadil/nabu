# Stack: todo-reminders

## Overview

Let users set **reminders on todos** and get a **push or email** at the chosen time. Flow: store reminders with a UTC fire time → a scanner claims due reminders → a delivery service sends them → users manage reminders through a JSON API. **v1 has no web UI** (the mobile app ships the picker).

## Slices

### 1. reminder-schedule
**Change:** `todo-reminders-1`
**Vertical slice:** Reminders are stored with a UTC fire time and a scanner claims due ones on a timer; operators can confirm scanner health (pending count, last scan).
**Responsibility:** `reminders` table; `ReminderScheduleService`; `ReminderScanner` with a claim step; `ReminderSink` interface with a logging sink; admin status and manual scan endpoints.
**Depends on:** None

### 2. reminder-delivery
**Change:** `todo-reminders-2`
**Vertical slice:** A due reminder reaches the user by push or email with the todo title, local time, and a deep link.
**Responsibility:** `ReminderDeliveryService` as the `ReminderSink`; `PushChannel` + `EmailChannel`; retry rule; stale token cleanup.
**Depends on:** None

### 3. reminder-api
**Change:** `todo-reminders-3`
**Vertical slice:** Users create, list, snooze, and cancel reminders on a todo. **POST** creates a pending reminder (optional test send); **GET** lists pending reminders.
**Responsibility:** `ReminderApiService`; local time + zone parsing; `POST` + `GET` + `PATCH` + `DELETE /todos/:todoId/reminders`; cascade cancel on todo delete; composite tests including the claimed-reminder conflict.
**Depends on:** `todo-reminders-1`, `todo-reminders-2`

## Implementation Order

1. `todo-reminders-1` — reminder schedule and scanner (parallel with #2)
2. `todo-reminders-2` — reminder delivery (parallel with #1)
3. `todo-reminders-3` — reminder API (create + list + snooze + cancel)

**Note:** `todo-reminders-4` was scaffolded during outline but **merged into slice 3** to keep one REST resource in a single vertical slice. Change directory `-4` remains empty until archived or repurposed.

## Cross-cutting Concerns

- **PR stacking:** Branches `feature/todo-reminders-1` … `-3` (slice 4 merged into 3); slice 3 must not merge without 1+2 on the same stack series.
- **Reuse:** Existing admin guard, Zod boundary parsing, push and email provider clients from sharing invites.
- **Own table:** Reminders must not live on the `todos` row (one todo, many reminders).
- **Slice 1 → 2 internal contract:** `ReminderSink.deliver(reminder)` with the claimed row. Slice 2 replaces the logging sink; nothing else calls the sink.
- **Slice 1 → 3 internal contract:** `ReminderScheduleService.create` and `findDue`. Slice 3 is the only user-facing consumer.
- **Slice 2 platform dependency:** Delivery reads `defaultChannel` and `devices` from the existing user model. Slice 2 has no code dependency on slice 1 beyond the sink interface.
- **Time:** Store UTC, keep the zone, convert at the boundary only.
- **v1 delivery:** One reminder => one message; no digests.
- **No-send contract:** Deleted or done todos never trigger a reminder.
- **Out of scope (unless pulled in):** Recurring reminders; location reminders; web UI.
