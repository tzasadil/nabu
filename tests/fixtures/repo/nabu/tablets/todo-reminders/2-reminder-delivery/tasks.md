# TODO Checklist

## Anchors

- `server/src/reminder-delivery/` — module (service, channels, renderer)
- `server/src/reminder-delivery/render-reminder.ts` — message body + deep link
- `server/src/reminder-delivery/reminder-delivery.service.ts` — load, pick channel, send, record
- `server/src/reminder-delivery/push-channel.ts` — push provider adapter
- `server/src/reminder-delivery/email-channel.ts` — email provider adapter
- `server/src/reminder-delivery/retry.ts` — transient vs permanent error rule
- `server/src/reminder-delivery/token-cleanup.ts` — drop dead device tokens

## 1. Pre-implementation gate

- [x] 1.1 Read `specs/reminder-delivery/spec.md` and `blueprint.md`; confirm slice 2 scope only (delivery). No user API or history.
- [x] 1.2 Confirm the `ReminderSink` interface from slice 1.
- [x] 1.3 Confirm `defaultChannel` and `devices` exist for every user.
- [x] 1.4 Confirm provider error codes for transient vs permanent.

## 2. Implement + verify

### 2.1 Types and renderer

- [x] 2.1.1 Define `DeliveryResult` and `DeliveryChannel` types.
- [x] 2.1.2 Implement `renderReminder` with title, local time, and deep link.

### 2.2 Tests first (red)

- [x] 2.2.1 Unit test: `renderReminder` shows the time in the stored zone.
- [x] 2.2.2 Unit test: retry rule classifies timeouts as transient and bad tokens as permanent.
- [x] 2.2.3 Composite test: push user gets one push; reminder becomes `sent`.
- [x] 2.2.4 Composite test: email user gets one email; reminder becomes `sent`.
- [x] 2.2.5 Composite test: user with no channel → `failed` with `no_channel`.
- [x] 2.2.6 Composite test: provider outage → reminder back to `pending`.
- [x] 2.2.7 Composite test: deleted todo → reminder cancelled, nothing sent.
- [x] 2.2.8 Composite test: done todo → nothing sent.
- [x] 2.2.9 Composite test: permanent push error deletes the token.
- [x] 2.2.10 Composite test: retries reuse the same idempotency key.
- [x] 2.2.11 Run new tests and confirm they fail before production implementation.

### 2.3 Channels and service

- [x] 2.3.1 Implement `PushChannel` and `EmailChannel` behind `DeliveryChannel`.
- [x] 2.3.2 Implement the retry rule.
- [x] 2.3.3 Implement `ReminderDeliveryService.deliver`: load → pick channel → render → send → record.

### 2.4 Wiring

- [x] 2.4.1 Register `ReminderDeliveryModule` as the `ReminderSink` provider.
- [x] 2.4.2 Remove the logging sink from the default wiring.
- [x] 2.4.3 Export `ReminderDeliveryService` for slice 3 reuse.

### 2.5 Validation

- [x] 2.5.1 `pnpm --filter server exec vitest src/reminder-delivery` passes.
- [x] 2.5.2 `pnpm --filter server build` passes for touched files.

## 3. Completion gate

- [x] 3.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 3.2 Reread the spec anew, then update it with new/missing insights from implementation. Ensure it doesn't contain implementation details, only contracts.
- [x] 3.3 Fill backlog.md with deferred out-of-scope issues. Remove already fixed.
