# Handoff: nabu

Goal: replace the upstream OpenSpec CLI with a small Rust CLI built for my own workflow. The tool is `nabu`. The schema keeps the name `specless`. Upstream's "change" is renamed to "tablet" (avoids a clash with jj). The folder layout changes to match.

## State

- Empty `cargo init` crate (edition 2024), colocated jj repo, no remote. It builds.
- Nothing designed or written yet. Start with design, not code.

## Constraints

| Topic | Decision |
|---|---|
| Users | Only me. No colleague compatibility needed. |
| Folder layout | New: `nabu/tablets/<name>/`, `nabu/archive/YYYY-MM-DD-<name>/`. A stacked tablet holds `stack.md` plus one subfolder per slice. Existing `openspec/` folders are migrated once. |
| Config | Global `~/.config/nabu/` holds settings and templates. A repo `nabu/` overrides only what differs. |
| Skills | Get templates from `nabu instructions`. Never embed a copy of the default template. |
| Backlog | Per-tablet `backlog.md`, non-normative, not counted in progress. |
| JSON output and command names | Free to change. The skills will be updated to match. |
| Upstream | `openspec` 1.14.3 is installed (`openspec --version`). None of its bundled skills are used. |
| Stack | Rust, `clap`, `serde`. Snapshot tests built from real `openspec/` folders. |

## Agreed MVP

| Command | Job |
|---|---|
| `new <change> [--stack <name>]` | Create the change folder from schema templates |
| `status [<change> \| --stack <name>]` | Which artifacts are done or blocked, per change or per stack |
| `instructions <artifact> <change>` | Template plus `config.toml` context and rules, as fed to the model |
| `list` | Active changes and stacks |
| `archive <change>` | Move to the dated archive. Replaces the plain `mv` in the archive skill. |

Dropped: `validate`. The `specless` schema has no spec deltas.

## What to read first

- Current CLI usage by my skills: `rg -n 'openspec (new|list|status|instructions|validate|archive)' ~/.claude/skills/openspec-*`
  - Calls today: `new change`, `list --json`, `status --change --json`, `instructions <artifact> --change --json`, `instructions build --change --json`, `validate --type change`.
- Skills to update after the tool exists: `~/.claude/skills/openspec-{apply,archive,learn,outline,propose,tweak,verify}/`
- Reference repo with a full history: `<reference repo>/openspec/`
  - Schema: `schemas/specless/schema.yaml` plus `templates/` (`blueprint.md`, `tasks.md`, `backlog.md`)
  - Config with context and per-artifact rules: `config.yaml`
  - Stack example: `stacks/todo-reminders/stack.md`
- Upstream behavior to compare against: run `openspec status --change <name> --json` and `openspec instructions <artifact> --change <name> --json` in the reference repo and capture the output as golden files.

## Open questions

- **Stacks:** what should `status --stack` report? Read `openspec-outline` and `openspec-propose` to see how stacks are created and consumed.
- **Templates:** does `instructions` keep upstream's output fields, or a leaner shape designed for the skills?

## Suggested skills

- `openspec-outline`: for designing this project itself and slicing it into reviewable steps.
- `writing-skills`: when updating the 7 `openspec-*` skills to call the new CLI.
- `debugging-duck` agent: before committing to the design, and before calling the MVP done.
- `workspace`: if working on this in parallel with another project.

## Ideas from upstream issues

Swept open issues of Fission-AI/OpenSpec on 2026-10-24. Ranked, size in brackets.

1. [S] #1907: `## Deferred` bullets in `tasks.md`. `archive` lists them before it finishes.
2. [S] #1084: `status` judges an artifact done by its content. An empty file or an unchanged template is not done.
3. [S] #1818: a draft state (proposal, no tasks). `list` shows drafts apart from active changes.
4. [M] #1915: a `depends_on` field in change metadata. `list` and `status` show order and which changes are ready.
5. [M] #1917: change ID = path. A directory with the metadata file is a change, others are namespaces. The same rule can find stack slices.
6. [M] #778, #1687: `extends: <parent>` schema, override single artifacts.
7. [S] #1323: `template` takes a filename or inline content.
8. [S] #1821, #835, #1524: global schemas, rules, and context in a user config. Project config overrides it.
9. [S-M] #1913, #1763, #1456: per-phase guidance (`new`, build, `archive`) in config, served by `instructions`.
10. [M] #1910, #704, #682: opt-in `beforeArchive` and `afterArchive` hooks. A failed hook fails the operation.
11. [S] #1983, #1142: `checks: {test, lint}` in config, passed to the agent through build instructions.
12. [S] #2006, #1652: `archive` refuses unchecked tasks. The error never suggests a bypass. Bypass = `--force`.
13. [S] #1245, #409: `created` and `archived` timestamps in metadata. Sort the archive by them.
14. [M] #2056: stable decision IDs (`D1`) that tasks cite. `status` flags references that do not resolve.
15. [S] #1831: `list` and `status` show "ready to archive".

Pain points to design around:
- A malformed config must cause a non-zero exit. (#1892)
- Find changes by the metadata file, not by `proposal.md`. (#1161, #666)
- One authoritative source for instructions, with one precedence rule. (#1893, #825)
- Resolve the root from the nearest folder => each jj workspace works on its own. (#1714)
- Overlap between parallel changes stays hidden until archive. (#1669, #1387)
