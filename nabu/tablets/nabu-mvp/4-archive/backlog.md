### Readiness guard

- **What:** refuse to archive while tracked tasks are unchecked, bypass with `--force` (upstream #2006).
- **Where:** `nabu archive`.
- **Why deferred:** not in MVP scope. Skills may check `nabu status` or `nabu list` first.
- **When to revisit:** a tablet gets archived with open tasks by mistake.
