# TODO Checklist

Working directory for every command: `/data/src/specless`.

## 1. Implement + test
- [x] 1.1 Create new `src/migrate.md` holding the `instruction`, `layout`, and `mapping` texts per their contracts. Compute the default hashes with `sha256sum` over `<reference repo>/openspec/schemas/specless/schema.yaml` and its `templates/` files.
- [x] 1.2 `nabu migrate` per `Header`, `Blocks`, `Exit`: a `Migrate` command in `src/main.rs` dispatched before `Layers::load` and `Config::load`, the prompt built with `src/prompt.rs` and the text file from 1.1.
- [x] 1.3 Binary tests asserting the header, the block tag order, and exit 0 (no full-prompt snapshot): a temp folder holding only `.jj/`; the fixture repo with a malformed repo config file (still exit 0).
- [x] 1.4 Dry run on throwaway copies, never the real repos. Build with `cargo build`, binary at `/data/src/specless/target/debug/nabu`.
  - Copy `<reference repo>/openspec/` plus an empty `.jj/` folder into a temp folder. Give the printed prompt and the binary path to a subagent. Tell it jj is not usable there, so step 1 of the prompt is skipped.
  - Pass when: `nabu list` prints a row for every former active change; the only error rows are unknown legacy schemas; `nabu status` exits 0 for every `specless` tablet; the repo schema and template copies are gone (hashes match); the refine sentences are dropped and reported; the reference repo's stack is one stacked tablet with its slices, and slices with no source folder are reported as skipped; the report lists unmapped items.
  - Repeat with `<variant repo>/openspec/`. Pass when its schema and `tasks.md` template stay as repo overrides, name `nabu-verify` instead of `openspec-verify`, and the report lists them.
  - Each miss is fixed in the prompt text from 1.1 or recorded in `backlog.md`.
  - Not checked: step 1 of the prompt (`jj new`, `jj op restore`), because jj is unusable in the throwaway copies. The variant-repo run used the prompt before the review fixes. The last reference-repo run used the final prompt.
- [x] 1.5 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. All pass.

## 2. Completion gate
- [x] 2.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 2.2 Fill backlog.md with follow up work. Remove already fixed. Update blueprint.md with insights from implementation. In `HANDOFF.md`, close the open question on migration (command vs script).
