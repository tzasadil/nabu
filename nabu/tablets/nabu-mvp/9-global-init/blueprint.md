# Blueprint nabu-mvp/9-global-init

## Feature description

Run `nabu init` once on this machine, so the nabu skills and agents are available in every repo. Nothing else outside this repo changes.

## Agent Decisions

- **State before the run** (recorded for rollback):
  - `~/.agents/skills/with-implementer` -> symlink to `/data/src/wiki/skills/with-implementer`. No `nabu-*` entries.
  - `~/.claude/skills` -> symlink to `~/.agents/skills` (left as is by init).
  - `~/.claude/agents/`: `artifacts.md`, `debugging-duck.md`, `Explore.md`, `implementer.md`, `openspec-opponent.md`, `openspec-reviewer.md`. `implementer.md` equals nabu's `agents/implementer.md`.
  - `~/.config/nabu/` does not exist.
- **Expected effect:** 8 skill folders in `~/.agents/skills/` (`nabu-*`, `with-implementer` as a real folder replacing the wiki link), 4 agents in `~/.claude/agents/` (`nabu-artifacts`, `nabu-opponent`, `nabu-reviewer`, `implementer` rewritten with identical content), a starter `~/.config/nabu/config.toml`.
- **Rollback:** `rm -r ~/.agents/skills/nabu-* ~/.agents/skills/with-implementer && ln -s /data/src/wiki/skills/with-implementer ~/.agents/skills/with-implementer`, `rm ~/.claude/agents/nabu-*.md`, `rm -r ~/.config/nabu`. `implementer.md` needs no rollback.

## User Decisions

- Only the global install. No repo migration, no wiki edit, no removal of old skills.
- The user confirms the run after seeing failure mode and rollback.

## Flow

- `cargo install --path .` -> state failure mode and rollback -> user confirms -> `nabu init` -> check effect.

## Contracts

- **Installed.** After the run, the expected effect holds exactly. Old `openspec-*` skills and agents are unchanged.
- **Usable.** `nabu` resolves on PATH, and a fresh Claude Code session lists the `nabu-*` skills.

## Out of scope

- Everything in the stack's "Cutover beyond the global install" item.

## Gotchas

- Running the wiki's `just link claude` later re-creates the `with-implementer` symlink over nabu's copy. Rerun `nabu init` afterwards, or retire the wiki copy in the later cutover.
