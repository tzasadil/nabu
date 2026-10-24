# TODO Checklist

Working directory for every command: `/data/src/specless`.

## 1. Implement + test
Binary tests run `CARGO_BIN_EXE_nabu` with `HOME` and `XDG_CONFIG_HOME` set to temp folders, as in earlier slices.
- [x] 1.1 Create new `build.rs` per the `Embedding by build script` and `Symlinks embedded as copies` decisions. Create new `src/starter-config.toml`. Unit test per `Embedding guard`.
- [x] 1.2 Dispatch `init` before `root::find_root` and the layers in `src/main.rs`. `nabu init` global per `Global`, `Installed content`, `Replace`, `Config`. Binary tests: run from a folder without `.jj/`; stdout of a fresh install has one line per embedded skill and agent and the config line (structural asserts, no full snapshot); `last-run.json` and `__pycache__` are not installed; scripts with a shebang are executable; `nabu-verify/implementer-rules.md` equals `with-implementer/implementer-rules.md`; a malformed existing global config is kept and init exits 0; fresh temp home gets all skills, agents, and a starter config that `nabu list` accepts; a pre-existing skill folder with an extra file and a pre-existing symlink skill are replaced, the symlink target unchanged; a foreign skill is untouched; an existing config is byte-identical after init; `~/.claude/skills` as a symlink to `~/.agents/skills` is left as is; missing `~/.claude/skills` becomes a link; a real `~/.claude/skills` folder is left with a notice; a read-only skills folder stops with exit 1 and the rerun fix.
- [x] 1.3 `nabu init --local` per `Local` and the link rule. Binary tests in a temp folder holding `.jj/`: fresh repo (link created, structural stdout asserts, `nabu list` accepts the new config, temp `HOME` unchanged); rerun replaces the installed skills; `.claude/skills` as a foreign symlink (left, notice printed); existing repo config kept.
- [x] 1.4 One binary snapshot per refusal in `Refusals`, and one for `--global --local`.
- [x] 1.5 Add a `nabu init` section to `README.md` (global vs `--local`, install from the checkout).
- [x] 1.6 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. All pass. Then `cargo install --path .`.

## 2. Completion gate
- [x] 2.1 Use skill `nabu-verify` and follow verbatim. Cannot be skipped or made up.
- [x] 2.2 Fill backlog.md with follow up work. Remove already fixed. Update blueprint.md with insights from implementation.
