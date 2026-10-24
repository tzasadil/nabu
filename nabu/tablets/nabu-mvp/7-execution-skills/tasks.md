# TODO Checklist

Working directory for every command: `/data/src/specless`.

## 1. Port
- [x] 1.1 Copy `/data/src/wiki/skills/openspec-apply/`, `/data/src/wiki/skills/openspec-verify/`, `/data/src/wiki/skills/openspec-archive/`, `/data/src/wiki/skills/openspec-learn/` (without `__pycache__`) to new folders under `skills/` named `nabu-build`, `nabu-verify`, `nabu-archive`, `nabu-learn`, and `/data/src/wiki/skills/with-implementer/` to new folder `skills/with-implementer`. Copy `/data/src/wiki/agents/openspec-reviewer.md` to `agents/` as `nabu-reviewer.md`, and the installed implementer agent from the home Claude agents folder to `agents/` as `implementer.md`. Never edit the sources.
- [x] 1.2 Rewrite every ported file per the blueprint decisions. Follow the `writing-skills` skill for every edit.
- [x] 1.3 Create the rule-writing reference of nabu-learn per the `Rule voice` decision, linked from learn's rule-drafting step.
- [x] 1.4 Switch the refine handoff of nabu-inscribe to `/nabu-build`. Leave nabu-tweak unchanged.
- [x] 1.5 Rename `openspec-verify` to `nabu-verify` in `defaults/schemas/specless/templates/tasks.md`. Update affected snapshots with `INSTA_UPDATE=always cargo test` and read each change. Run `cargo install --path .`.

## 2. Verify
- [x] 2.1 Run the two `rg` checks from `No OpenSpec coupling` and `No template text`. Only allowlisted matches.
- [x] 2.2 `uv run -q --no-project --with pyyaml python /data/src/wiki/skills/writing-skills/scripts/validate_skills.py skills`: 8 skills, 0 errors (wiki present here). `diff` the agents against their sources: only allowed changes.
- [x] 2.3 Run each command in `Scripts` and `Learn tooling`. `nabu instructions tasks nabu-mvp/7` shows `nabu-verify`. `test -f skills/nabu-verify/implementer-rules.md` (symlink resolves). Check the `Handoffs` contract with `rg`. Not checked: mining run phases, this repo has no session that loaded a `nabu-*` skill yet (0 sessions; findings carry label `unknown`). Regex checked on sample strings instead. `diff-scan lines` ran on a temp copy at slice 6; `full` keeps `nabu/**` paths by design.
- [x] 2.4 Archive flow dry run on a plain temp copy of the fixture repo (`tests/fixtures/repo`): follow the ported nabu-archive steps on the stacked fixture tablet and a slice target. Record what the skill asks and the resulting tree. Hand run, not an agent run. Observed: asks 1) archive the whole tablet `todo-reminders` instead of the slice, 2) backlog dispositions for all 3 slice backlogs plus the one approval. Slice target refused: `error: cannot archive "todo-reminders/1-reminder-schedule": archive moves whole tablets` / `fix: nabu archive todo-reminders`. Archive ran first, then the item moved to new tablet `reminder-delivery-retry` (rerun in the final order, same tree). Result: `nabu/archive/2026-10-24-todo-reminders/` with slices 1, 2, 3; active `42-empty-list-notice` and `reminder-delivery-retry`; a second archive attempt fails with `unknown tablet`.
- [x] 2.5 `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`: pass. `cargo test` ran once, together with the snapshot update in 1.5.

## 3. Completion gate
- [x] 3.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up. For its Phase 4 squash, use `nabu/tablets/nabu-mvp/stack.md` and `--keep-emptied`.
- [x] 3.2 Fill backlog.md with follow up work. Remove already fixed. Update blueprint.md with insights from implementation.
