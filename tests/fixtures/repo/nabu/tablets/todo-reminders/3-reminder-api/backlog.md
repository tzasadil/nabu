<!---
Deferred findings for this change: **not** part of the delta spec contract (`specs/**/*.md`).
-->

### Sent reminder retention

- **What:** Prune `sent` and `failed` reminder rows; v1 keeps every row while GET lists pending only.
- **Where:** `reminders` table
- **Why deferred:** history API not in v1; acceptable table growth until operator feedback.

### POST idempotency

- **What:** Idempotency-Key guard so a double tap does not create two reminders.
- **Where:** `ReminderController` POST
- **Why deferred:** v1 accepts duplicates; clients can debounce.

### Live scanner state in the response

- **What:** Report whether the scanner is running when a reminder is created.
- **Where:** `ReminderApiService`
- **Why deferred:** in-process status is enough for single-instance v1; slice 1 blueprint documents the multi-instance caveat.

### Partial success on bulk create

- **What:** Create the valid reminders and report the invalid ones instead of failing the whole bulk request.
- **Where:** `ReminderApiService`
- **Why deferred:** simpler failure contract for v1; all-or-nothing specified in spec.

### Snooze presets

- **What:** Accept `snooze: "1h" | "tomorrow"` instead of an explicit new time.
- **Where:** PATCH handler
- **Why deferred:** stack v1 takes an explicit time so clients own the presets.


- **What:** `GET /todos/:todoId/reminders/:reminderId/attempts` and a history list.
- **Where:** `ReminderController`
- **Why deferred:** v1 GET returns pending reminders only; `reminderId` is stable for later use.

### Local time on the response

- **What:** Return `fireAtLocal` next to `fireAt`.
- **Where:** Reminder response DTO
- **Why deferred:** Clients format time themselves in v1.

### Reminder count cap per todo

- **What:** Refuse a new reminder when a todo already has N pending.
- **Where:** `ReminderApiService`
- **Why deferred:** until real usage data exists.

### POST rate limiting

- **What:** Per-user throttle on POST to limit reminder spam.
- **Where:** `ReminderController` POST
- **Why deferred:** v1 relies on auth + the delivery cap backlog item; add when abuse data warrants.

### Bulk snooze

- **What:** Snooze every reminder on a list in one call.
- **Where:** Reminder module
- **Why deferred:** single snooze is enough for v1.

### Web reminder picker

- **What:** UI to add, snooze, and cancel reminders.
- **Where:** `web/src/todos`
- **Why deferred:** stack v1 is the JSON API; the mobile app ships the picker first.
