# Blueprint nabu-mvp/8-init

## Feature description

`nabu init` installs the nabu skills and agents, embedded in the binary, so they always match the CLI that serves them.

- `nabu init` (same as `nabu init --global`): installs into the user's home and writes a starter `~/.config/nabu/config.toml` when missing.
- `nabu init --local`: installs into the current repo only and writes a starter `nabu/config.toml` when missing. Nothing outside the repo changes.

## Agent Decisions

- **Embedding by build script.** `build.rs` walks `skills/` and `agents/` and generates the list of `(path, bytes)` entries. It skips runtime state: `__pycache__/`, `*.pyc`, and `nabu-learn/last-run.json` (learn runs its repo copy and keeps its cursor there, slice 7). The rule lives in `src/runtime_state.rs`, loaded by `build.rs` through `#[path]` and unit tested. An entry that is neither file nor folder (dangling link) fails the build. A hand list would drift on every skill edit. The build reruns when either folder changes.
- **Symlinks embedded as copies.** `skills/nabu-verify/implementer-rules.md` links to `../with-implementer/implementer-rules.md`. It is embedded with its target's bytes => an installed skill never depends on a link that may not survive the install.
- **Executable bit from the shebang.** An installed file starting with `#!` gets mode `0o755`. That covers the 5 executable scripts and also the sourced `lib.sh`, which is harmless. No mode tracking in the build.
- **Skill install = replace.** For each embedded skill, the target `<skills dir>/<name>` is removed (a symlink is unlinked, never followed; a folder is removed with its contents) and written fresh. No merge => files dropped from a skill never linger. Agents are single files, overwritten.
- **Dispatch first.** `init` dispatches before `root::find_root`, `Layers::load`, and `Config::load` (even earlier than `migrate`, which needs the root). Global needs no root. `--local` calls only `root::find_root`. A malformed config never blocks the repair rerun.
- **Global targets.** Skills: `~/.agents/skills/`. Agents: `~/.claude/agents/`. `~/.agents/agents/` is not used: no harness on this machine reads it. Config path: `root::global_dir`, the function the layers read. `HOME` is required for all of global init, checked once before any write.
- **`.claude/skills` link rule, global and local.** Missing -> symlink to the installed skills folder (`~/.agents/skills`, or `../.agents/skills` locally). Anything else -> left as is, with one notice when it does not resolve to the installed skills folder. Real-folder merging is in the backlog.
- **Local targets.** Skills: `<root>/.agents/skills/`. `.claude/skills` per the link rule. Agents: `<root>/.claude/agents/`. Root = slice 1 root rule. The local config file is a root marker: init in a subfolder of a jj repo still targets the root the rule finds.
- **No writes through links.** A target folder that is a symlink is never written through. `--local` refuses when `<root>/.agents/skills`, `<root>/.claude/agents`, or any of `<root>/.agents`, `<root>/.claude`, `<root>/nabu` is a symlink (in this repo the first two link to the sources): a linked parent would let init write or delete outside the repo. Global refuses only the leaves `~/.agents/skills` and `~/.claude/agents`: a user-managed `~/.agents` or `~/.claude` link (dotfiles) is intended, and the home folder is the install target anyway.
- **Stop at the first error.** An I/O error mid-install exits 1 with `error:` naming the path and `fix: rerun nabu init`. Install is idempotent, so a rerun heals a half-written skill.
- **Config never overwritten.** Starter config is written only when the file is missing. Its text lives in `src/starter-config.toml` (next to the code, not under `defaults/`, which holds only built-in layer files; one source for global and local): comments only, one per key (`schema`, `context`, `[rules]`), no values => the built-in layer keeps deciding.
- **Output.** One line per installed skill and agent (the folder or file path), root-relative for `--local`, absolute for global, then notices, then a last line naming the config file and whether it was written or kept. `--global` and `--local` conflict (clap usage error).

## User Decisions

- `nabu init` defaults to global. `--local` is a fully project-local setup: nothing in global config or home folders.
- Skills and agents are embedded in the binary => installed copies match the CLI.
- Running again overwrites. No merge logic. Undo for `--local` is `jj op restore`. Global installs have no VCS: rerun an older `nabu init` to go back.
- Starter configs only, no template copies.
- Local projects still read the global layer.

## Flow

- `nabu init [--global]` -> resolve home (`HOME`) -> refuse if `~/.agents/skills` or `~/.claude/agents` is a symlink -> for each skill: replace in `~/.agents/skills/` -> `~/.claude/skills` link rule -> agents into `~/.claude/agents/` -> starter config if missing -> print.
- `nabu init --local` -> find root -> refuse if `.agents/skills` or `.claude/agents` is a symlink -> skills into `.agents/skills/` -> `.claude/skills` link rule -> agents into `.claude/agents/` -> starter `nabu/config.toml` if missing -> print.

## Contracts

- **Installed content.** Every embedded file lands byte-identical under its skill or agent name, symlinks as copies of their target. Executable sources stay executable.
- **Replace.** A pre-existing `<skills dir>/<name>` (folder with extra files, or a symlink to elsewhere) is fully replaced. The symlink target is untouched. Skills nabu does not ship are untouched.
- **Global.** Writes only under `$HOME/.agents/skills/`, `$HOME/.claude/skills` (only the link, per the rule), `$HOME/.claude/agents/`, and the global config file. Missing folders are created. Works from any folder, repo or not.
- **Local.** Writes only under `<root>/.agents/skills/`, `<root>/.claude/`, and `<root>/nabu/config.toml`. `HOME` stays untouched. `.claude/skills` follows the link rule. The global layer still applies to the repo afterwards.
- **Config.** An existing config file is never modified, even when malformed. A written starter config parses as valid nabu config.
- **Refusals** (stderr, exit 1, `error:` then `fix:`, nothing written). A symlinked target folder per `No writes through links`. `--local` outside any nabu or jj root (slice 1 root error). Global with `HOME` unset.
- **Embedding guard.** A test walks `skills/` and `agents/` with `std::fs` independently of `build.rs` and compares paths with the embedded list, minus the skipped runtime state. The symlink entry equals its target's bytes.

## Out of scope

- Version stamps in installed skills and stale-copy warnings (stack Out of scope).
- Removing skills nabu no longer ships.
- Agent folders of other harnesses (`~/.config/opencode/agents`).
- Running `nabu init` on this machine for real (slice 9).

## Gotchas

- `cargo install --path .` after this slice: the installed binary must carry the embedded skills before slice 9 runs `nabu init`.
- Global install replaces a same-named wiki symlink (for example `with-implementer`). Intended at cutover, so slice 9 runs it, not this slice.
- Tests must point `HOME` at a temp folder: a wrong `HOME` would replace the user's real skills.
- `cargo install` and `cargo package` now need `skills/` and `agents/` beside `Cargo.toml`: install from the repo checkout.
- `init` is unix-only (`PermissionsExt`, symlinks). Fine for this machine; a Windows port needs another link strategy.
- Creating the `.claude/skills` link prints no line. A dangling symlink at that path counts as present: notice, no link.
- Existing agent files, symlinks included, are removed before writing => a write never goes through a link.
