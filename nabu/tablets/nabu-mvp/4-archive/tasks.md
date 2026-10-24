# TODO Checklist

Working directory for every command: `/data/src/specless`.

## 1. Implement + test
Binary tests run `CARGO_BIN_EXE_nabu` in a temp copy of `tests/fixtures/repo`, as in earlier slices, with `insta` snapshots filtering the date to `[DATE]`.
- [x] 1.1 Add the dependency with `cargo add jiff`.
- [x] 1.2 `nabu archive <tablet>` per the `Move` contract. Binary tests: the fixture stacked tablet moves with `stack.md` and all slices, content identical; the two-zone local-date test per the `Tests fix the date by filter` decision; `nabu/archive/` is created when missing (the test removes it from the temp copy first); a tablet whose `.nabu.toml` names an unknown schema still archives.
- [x] 1.3 One binary snapshot per refusal, each leaving the tree unchanged: slice target, existing target folder (the test runs with `TZ=Etc/GMT-14` and pre-creates folders for that zone's dates read before and after the run), unknown tablet. Rename failure itself is not tested: it cannot be forced once the target checks pass.
- [x] 1.4 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. All pass.

## 2. Completion gate
- [x] 2.1 Use skill `openspec-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 2.2 Fill backlog.md with follow up work. Remove already fixed. Update blueprint.md with insights from implementation.
