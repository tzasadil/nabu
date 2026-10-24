# TODO Checklist

Working directory for every command: `/data/src/specless`.

## 1. Built-in defaults
- [x] 1.1 Add dependencies with `cargo add clap --features derive`, `cargo add serde --features derive`, `cargo add toml`, `cargo add --dev insta tempfile`.
- [x] 1.2 Create new folder `defaults/` holding the built-in layer: `config.toml` with `schema = "specless"`, and `schemas/specless/` with `schema.toml` and `templates/`.
- [x] 1.3 Built-in `schema.toml`: convert `<reference repo>/openspec/schemas/specless/schema.yaml` to the blueprint `File keys`. Drop `version`, drop `backlog` from `tasks.requires`, add `stack = "stack.md"`, reword "change" to "tablet".
- [x] 1.4 Built-in `templates/`: `blueprint.md`, `backlog.md`, `tasks.md` from `<reference repo>/openspec/schemas/specless/templates/` and `stack.md` from `~/.claude/skills/openspec-outline/templates/stack.md`. Reword "change" to "tablet".

## 2. Fixtures
- [x] 2.1 Create new folder `tests/fixtures/` with a repo fixture in nabu layout. Each `.openspec.yaml` becomes `.nabu.toml` with `schema = "specless"`. `specs/` folders are dropped.
  - plain tablet from `<reference repo>/openspec/changes/42-empty-list-notice/`
  - stacked tablet from `<reference repo>/openspec/stacks/todo-reminders/stack.md` plus `<reference repo>/openspec/changes/todo-reminders-1/`, `-2/`, `-3/` as slices numbered 1 to 3
  - archived tablet from `<reference repo>/openspec/changes/archive/2026-08-14-split-sync-worker/`
- [x] 2.2 Add a repo-layer override fixture: `<variant repo>/openspec/schemas/specless/schema.yaml` converted to TOML without a `stack` key, plus only `<variant repo>/openspec/schemas/specless/templates/blueprint.md` as its single template.

## 3. Implement + test
Binary tests run `CARGO_BIN_EXE_nabu` in a temp copy of a fixture, with `HOME` and `XDG_CONFIG_HOME` set to temp folders. Snapshot output and errors with `insta`, filtering the temp prefix. Logic no slice 1 command prints (config merge, template resolution, lookup, embedding) gets in-crate unit tests. No debug subcommand.
- [x] 3.1 Root and global path per `Root` and `Global layer path` contracts. Tests: nabu marker found from a nested cwd, marker beats a nearer `.jj/`, `.jj/` fallback, no-root error, empty `XDG_CONFIG_HOME` falls back to `$HOME/.config/nabu`.
- [x] 3.2 Layers and config per `Layers`, `Config merge`, `File keys` contracts. Unit tests: context and rules appended global first, `schema` nearest wins. Binary tests: malformed TOML names the file, unknown key names the file.
- [x] 3.3 Built-in embedding per the `Embedding` decision. Unit test: every file under `defaults/` is embedded.
- [x] 3.4 Schemas per `Schema resolution`, `Schema validity`, `Built-in schema` contracts. Unit tests: repo override schema wins, `blueprint.md` from the override and other templates from the built-in. Binary tests: zero-config `new` works, unknown schema lists the available ones, missing required key, unresolvable stack template.
- [x] 3.5 Names and lookup per `Name rule` and `Lookup`. Unit tests: each lookup form, unknown slice lists slices. Binary tests: `new` under an unknown tablet lists tablets without archived ones, invalid tablet and slice names.
- [x] 3.6 `nabu new` per `new tablet`, `new --stack`, `new slice`, `Refusals`, `Errors`. Tests: each form, slice numbering after the fixture's 3 slices, each refusal leaves the tree unchanged, `--stack` under the override schema refuses.
- [x] 3.7 Composite test: in an empty temp folder holding only `.jj/`, run `nabu new demo --stack`, `nabu new demo/first`, `nabu new demo/second`, then snapshot the resulting tree and file contents.
- [x] 3.8 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. All pass.

## 4. Docs
- [x] 4.1 Update `AGENTS.md` Design invariants: config and schema files are TOML, three layers including the built-in one.
- [x] 4.2 Update `HANDOFF.md` Agreed MVP table: the `instructions` row names the TOML config file instead of the YAML one. The reference-repo mention stays, it describes OpenSpec.

## 5. Completion gate
- [x] 5.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 5.2 Fill backlog.md with follow up work. Remove already fixed. Update blueprint.md with insights from implementation.
