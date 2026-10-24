# TODO Checklist

Working directory for every command: `/data/src/specless`.

## 1. Implement + test
Binary tests run `CARGO_BIN_EXE_nabu` in a temp copy of `tests/fixtures/repo`, as in slices 1 and 2, with `insta` snapshots. Test-specific files are written into the temp copy by the test.
- [x] 1.1 `generates` check in `schema::resolve` (`src/schema.rs`). Unit test: each bad form names the schema file and the value.
- [x] 1.2 Rules keep their source file through the config merge (`src/config.rs`). Rules-key check per tablet schema per the `Rules keys` decision. Rules sources as a sibling map in `src/config.rs`, existing tests unchanged. Unit tests: artifact-id keys pass; an unknown key names every config file that sets it.
- [x] 1.3 Block and header formatting per `Block format` and `List values in the header`. Extend the task parser in `src/report.rs` to return unchecked lines. Unit tests: empty block omitted, surrounding blank lines trimmed, context joined per `Context block`, `none` for empty lists, template-equal tracks file gives no pending lines.
- [x] 1.4 `nabu instructions <artifact> <target>` per `Artifact prompt` and the `Read list` decision. Binary snapshots, following the `Artifact prompt` header order: `tasks` for a fixture slice, `blueprint` for the fixture plain tablet with global and repo context and rules written into the temp layers, a blocked artifact.
- [x] 1.5 `nabu instructions build <target>` per `Apply prompt`. Binary snapshots: a fixture slice with pending tasks, a slice whose tasks are all checked (`all done`), a slice without `tasks.md` (`blocked`).
- [x] 1.6 `nabu instructions stack <tablet>` per `Stack prompt`. Binary snapshots: the fixture stacked tablet, a plain tablet (state `ready`).
- [x] 1.7 One binary snapshot per new refusal: unknown artifact id, stacked root target, `stack` with a slice target, `stack` with a schema lacking `stack`, bad rules key on `apply`.
- [x] 1.8 Drop the remaining `#[allow(dead_code)]` markers in `src/config.rs` and `src/schema.rs` once their fields are read.
- [x] 1.9 Composite test: in an empty temp folder holding only `.jj/`, run `nabu new demo --stack`, `nabu new demo/first`, `nabu instructions blueprint demo/first`. Assert its `write:` path is the folder `new` created and `state: ready`. Write the blueprint, rerun, assert `state: done`.
- [x] 1.10 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. All pass.

## 2. Completion gate
- [x] 2.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 2.2 Fill backlog.md with follow up work. Remove already fixed. Update blueprint.md with insights from implementation.
