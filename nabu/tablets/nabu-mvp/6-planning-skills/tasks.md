# TODO Checklist

Working directory for every command: `/data/src/specless`.

## 1. Port
- [x] 1.1 Copy `/data/src/wiki/skills/openspec-outline/`, `/data/src/wiki/skills/openspec-propose/`, `/data/src/wiki/skills/openspec-tweak/` (reviewers and scripts included) to new folders under `skills/` named `nabu-outline`, `nabu-inscribe`, `nabu-tweak`. Create new folder `agents/` with copies of `/data/src/wiki/agents/openspec-opponent.md` and `/data/src/wiki/agents/artifacts.md`, named `nabu-opponent.md` and `nabu-artifacts.md`. Never edit the wiki copies.
- [x] 1.2 Rewrite every ported file, reviewers included, per the blueprint decisions: names, CLI calls, paths, the stop rule, plain-tablet `nabu new`, CLI failure handling, the jj stack sequence, the `--keep-emptied` squash, `read:` instead of `contextFiles`. Delete the outline `templates/` folder. Follow the `writing-skills` skill for every edit.
- [x] 1.3 Port the bundle check per the `Bundle check for nabu` decision and replace `openspec validate` in the refine procedure with it.
- [x] 1.4 Create the three project links per the blueprint.
- [x] 1.5 Run `cargo install --path .` so skills can call `nabu`.

## 2. Verify
- [x] 2.1 `rg -in 'openspec|skip_specs' skills/nabu-outline skills/nabu-inscribe skills/nabu-tweak agents`: only the allowlisted mention. `rg -n '^# (Stack:|Blueprint|TODO Checklist)' skills`: no match.
- [x] 2.2 `uv run -q --no-project --with pyyaml python /data/src/wiki/skills/writing-skills/scripts/validate_skills.py skills`: 3 skills, 0 errors. Agent frontmatter differs from the wiki sources only in `name` and `description`.
- [x] 2.3 Bundle check: exit 0 on `nabu/tablets/nabu-mvp/5-migrate`; exit 2 on `nabu/tablets/nabu-mvp/9-cutover` (no `tasks.md`). In a temp copy of the 5-migrate folder with one unresolved task path and one uncertainty marker outside Gotchas: exit 1 naming both.
- [x] 2.4 Throwaway jj repo (`jj git init` in a temp folder) with one unrelated edit in `@`: run the skills' exact outline sequence (stack revision, `nabu new --stack`, `nabu new` per slice, jj chain). Assert the `Stack in jj` contract with `jj log` and `jj diff --summary`. Also run one propose sequence on a plain tablet up to `state: ready`. Read-only commands (`status`, `list`, `instructions`) may also run on this repo's `nabu-mvp` tablet. Record the commands run. Commands run in `mktemp -d`: `jj git init`; `jj new -m "demo: stack"` (@ held `notes.txt`); `nabu new demo --stack`; `nabu instructions stack demo`; `nabu new demo/{alpha,beta,gamma}`; `jj new -m "demo N: <slice>"` x3; `jj diff --summary -r <rev>` per revision (stack: `stack.md` + 4 markers, parent: `notes.txt`, slices: empty); `nabu new plain`; `nabu status plain`; `nabu instructions build plain` (blocked, then `state: ready` after blueprint and tasks); `check_bundle.py` (exit 0); `jj squash -u --keep-emptied --from @ --to <stack-rev> nabu/tablets/demo/stack.md` (slice revision stays, empty). Not checked: skill prose was not run through a fresh agent session (writing-skills "Testing"), the commands were replayed by hand.
- [ ] 2.5 `test -f .claude/skills/nabu-outline/SKILL.md && test -f .claude/agents/nabu-opponent.md`. Ask the user to confirm in a fresh Claude Code session here that `/agents` lists `nabu-opponent`. Record the answer and settle the UNCERTAIN gotcha. Not checked: the `/agents` listing in a fresh session, needs the user. The two `test -f` checks pass.

## 3. Completion gate
- [x] 3.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up. For its Phase 4 squash, use `nabu/tablets/nabu-mvp/stack.md` and `--keep-emptied`.
- [x] 3.2 Fill backlog.md with follow up work. Remove already fixed. Update blueprint.md with insights from implementation.
