# Blueprint todo-reminders-2

## Decisions

- **Module `reminder-delivery`** under `server/src/reminder-delivery/` — implements the `ReminderSink` from slice 1. Register it in the app module so it replaces the logging sink.
- **Two channels** — `PushChannel` (device tokens from `devices`) and `EmailChannel` (user email). The user's `defaultChannel` picks one. No per-reminder channel in v1.
- **Render once** — One message body per reminder: todo title, local fire time, deep link to the todo. Same text for push and email.
- **Per-device fan-out** — REMOVED; push goes to the provider topic for the user, not to each device.
- **Retry rule** — Retry transient provider errors up to 3 times with backoff. Permanent errors (bad token, bounced address) mark the reminder `failed` without retry.
- **Stale token cleanup** — A permanent push error for a token deletes that token from `devices`.
- **Errors** — A user with no device and no email → reminder `failed` with reason `no_channel`. Provider outage → back to `pending` for the next tick.
- **No new tables** — State lives on the `reminders` row from slice 1 (`sent`, `failed`, `failureReason`).
- **Export service for slice 3** — Export `ReminderDeliveryService.sendNow(reminderId)` so slice 3 can offer "send test reminder" without a scan.

## Flow

1. Scanner claims a due reminder → `ReminderDeliveryService.deliver`
2. Load the todo and the user
3. Pick the channel from `defaultChannel`
4. Render the message → send through the channel
5. Success → `sent`; transient error → retry; permanent error → `failed`
6. Update the reminder row

## Boundaries

- **In scope:** `ReminderDeliveryService`, `PushChannel`, `EmailChannel`, message rendering, retry rule, token cleanup, composite tests with fake providers.
- **Out of scope:** Scanner and table (`todo-reminders-1`), user API and history (`todo-reminders-3`), UI, digest emails, per-reminder channel choice.
- **Assumption:** Users already have a `defaultChannel` and registered devices from the existing sharing invites flow.

## Gotchas

- **Deleted todos** — A reminder whose todo was deleted must be cancelled, not sent.
- **Completed todos** — Do not send a reminder for a todo already marked done.
- **Time zone in the body** — Render the fire time in the reminder's stored zone, not the server zone.
- **Provider timeouts** — Count a timeout as transient; never as success.
- **Duplicate sends** — The claim from slice 1 prevents two scanners sending the same reminder; retries must reuse the same provider idempotency key.

## Open questions

- Whether to add `REMINDER_PUSH_KEY` or reuse `PUSH_API_KEY` for v1 (default: reuse).
