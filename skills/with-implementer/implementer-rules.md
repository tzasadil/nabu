# Implementer rules

Learned from past sessions. Hold in every repo.

## UI

Apply when the batch touches user-facing code. Each line is a defect the verify ui lane kept finding after implementation.

- Extract shared components and reuse them aggressively -> pages stay visually consistent.
- UI copy: short label and bare value, not sentences. Cut caveats, mechanics, and technical detail.
- Newest async result wins. A stale response never overwrites newer state.
- Awaited actions leave their busy state on failure or timeout.
- Disabled controls explain why on hover and on keyboard focus. Native `disabled` swallows both: wrap the control or use `aria-disabled`.
- Async changes and errors announce through an always-mounted live region. A region mounted with its content is not announced reliably. Errors use `role="alert"`.
- Focus moves to a stable target when the focused control unmounts or is replaced (Add, Remove, Retry, dialog close). It never drops to the page body.
- Failures show the client's own copy. Raw transport, HTML, or parser error text never reaches the user.
- Confirmation popup for destructive actions.
- Paginate unbounded DB lists shown to users.
