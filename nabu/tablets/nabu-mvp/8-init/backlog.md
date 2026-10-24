### Stale-copy warning

- **What:** stamp the nabu version into installed skills; warn when an installed copy differs from the binary.
- **Where:** `nabu init`, maybe `nabu status`.
- **Why deferred:** stack Out of scope. Both copies come from one binary when refreshed together.
- **When to revisit:** first confusion from a stale local copy.

### Remove dropped skills

- **What:** `nabu init` removes `nabu-*` skills it no longer ships.
- **Where:** `nabu init`.
- **Why deferred:** no skill has been dropped yet.
- **When to revisit:** the slimming stack renames or merges a skill.

### Real `.claude/skills` folders

- **What:** when `.claude/skills` is a real folder, add one link per nabu skill instead of only printing a notice.
- **Where:** `nabu init` link rule, global and local.
- **Why deferred:** no observed home or repo has a real `.claude/skills` folder.
- **When to revisit:** the first project or machine that does.
