# TODO Checklist

Working directory for every command: `/data/src/specless`.

## 1. Implement + test
Binary tests run `CARGO_BIN_EXE_nabu` in a temp copy of `tests/fixtures/repo`, as in slice 1, with `insta` snapshots. Test-specific files (template-equal artifacts, broken schemas) are written into the temp copy by the test.
- [x] 1.1 Graph checks per the `Graph checks` contract, inside `schema::resolve` in `src/schema.rs`. Unit tests: one per error, a cycle names its ids. Binary test: `status` on a tablet whose repo schema has a cycle.
- [x] 1.2 Move the target split and the stacked check into `src/tablets.rs`, make `layers::read_file` crate-visible, per the `Reuse slice 1 owners` decision. Existing `new` tests stay green.
- [x] 1.3 Artifact state, progress, and reads per `Artifact state`, `Artifact order`, `Progress`, `Reads`, `Ready to archive`. Unit tests: missing, empty, whitespace-only, template-equal including CRLF and trailing whitespace, edited; mixed `requires` lists only the unfinished ids; checkbox forms (`-`, `*`, `+`, nested, `[X]`); missing tracks file is `0/0`; template-equal tracks file is `0/0`; non-UTF-8 tracks file is an error naming it; zero tasks never ready; stacked tablet with zero slices or one unfinished slice not ready.
- [x] 1.4 `nabu status <tablet>` and `<tablet>/<slice>` per `Table rule` and the three output contracts. Binary snapshots: the fixture plain tablet, the fixture stacked tablet, one slice target, a slice whose `tasks.md` equals the template, a stacked tablet with a stray `blueprint.md` at its root (ignored), a plain tablet with a slice-shaped subfolder (ignored), a slice target on a plain tablet, a target with more than one `/`, one unknown slice.
- [x] 1.5 `nabu list` per `List output` and `List failures`. Change the command outcome in `src/main.rs` to stdout text plus exit code. Binary snapshots: the fixture repo (archived tablet absent), an empty repo, a repo with one broken `.nabu.toml` tablet and one broken-graph tablet among good ones (exit 1, other rows still printed, stderr empty).
- [x] 1.6 Narrow the `#[allow(dead_code)]` markers per the `Dead-code allows per field` decision in `src/schema.rs`, `src/tablets.rs`, `src/config.rs`.
- [x] 1.7 Composite test: in an empty temp folder holding only `.jj/`, run `nabu new demo --stack`, `nabu new demo/first`, write a done blueprint and a `tasks.md` with one checked task into `first`, then snapshot `nabu status demo` and `nabu list`.
- [x] 1.8 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. All pass.

## 2. Completion gate
- [x] 2.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 2.2 Fill backlog.md with follow up work. Remove already fixed. Update blueprint.md with insights from implementation.
