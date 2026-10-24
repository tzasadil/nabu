<!---
Deferred findings for this change: **not** part of the delta spec contract (`specs/**/*.md`).
-->

### Dedicated push credentials env

- **What:** `REMINDER_PUSH_KEY` separate from the general push key for rate limits.
- **Where:** `server/src/config/settings.ts`
- **Why deferred:** v1 reuses `PUSH_API_KEY`; add when operators need separate limits.

### Persist delivery attempts

- **What:** Store each delivery attempt per reminder for support questions.
- **Where:** New table + `ReminderDeliveryService`
- **Why deferred:** slice 3 owns reminder history; attempt logs are support tooling, not v1 contract.

### Batch email digest

- **What:** Group reminders due in the same minute into one email.
- **Where:** `EmailChannel`
- **Why deferred:** one email per reminder is fine for v1; add if users complain about volume.

### Per-user delivery cap

- **What:** Cap reminders sent per user per hour (e.g. 50) and drop the rest with a log line.
- **Where:** `ReminderDeliveryService`
- **Why deferred:** v1 relies on provider rate limits; add when abuse data warrants.

### Shared notification channel module

- **What:** Extract `PushChannel` + `EmailChannel` into a shared module used by sharing invites too.
- **Where:** `server/src/shared/notify/`
- **Why deferred:** slice 2 vertical slice; dedupe in follow-up refactor.


- **What:** Settings panel to pick push, email, or both per reminder.
- **Where:** `web/src/settings`
- **Why deferred:** stack v1 uses the user's default channel only.
