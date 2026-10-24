# TODO Checklist

Working directory for every command: `/data/src/specless`.

## 1. Install
- [x] 1.1 Run `cargo install --path .`.
- [x] 1.2 Show the user the failure mode and the rollback from the blueprint. Get explicit confirmation.
- [x] 1.3 Run `nabu init`.

## 2. Check
- [x] 2.1 Done 2026-10-24: 8 skills, 4 agents, starter config written; `nabu-inscribe` and `implementer.md` identical to the sources. `ls -la ~/.agents/skills ~/.claude/agents ~/.config/nabu` matches the `Installed` contract. `diff -r skills/nabu-inscribe ~/.agents/skills/nabu-inscribe` shows no difference.
- [ ] 2.2 Ask the user to confirm in a fresh Claude Code session that the `nabu-*` skills and `nabu-opponent` agent are listed. This also settles slice 6 task 2.5.

## 3. Completion gate
- [ ] 3.1 Use skill `nabu-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 3.2 Fill backlog.md with follow up work. Remove already fixed. Update blueprint.md with insights from implementation.
