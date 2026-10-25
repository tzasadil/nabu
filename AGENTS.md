# AGENTS.md

`nabu`: Rust CLI for spec-driven work. Replaces the OpenSpec CLI for one user. Goals: `README.md`. Current state and open questions: `HANDOFF.md`.

## Design invariants

- jj native. Git optional => no git calls. No commits as a side effect.
- Term: tablet = one unit of planned work (upstream: change). "Change" means a jj revision only.
- Layout: `nabu/tablets/<name>/`, archive `nabu/archive/YYYY-MM-DD-<name>/`.
- Stacks first class. Tablet with `stack.md` -> one subfolder per slice. Every command handles plain and stacked tablets.
- Configurable. Artifacts, templates, rules come from schema + `config.toml`. No artifact names hard-coded in command logic.
- Config and schema files are TOML (`config.toml`, `schema.toml`, `.nabu.toml`). Prose is inline.
- Config layers: built-in (embedded from `defaults/`) -> global `~/.config/nabu/` -> repo `nabu/`. Nearer layer overrides.
- Templates have one source. Skills fetch them with `nabu instructions` => no template text copied into skills.
- Backlog: per-tablet `backlog.md`, non-normative. Never counted in progress.
- Lean artifacts. Templates ask for intent, contracts, decisions. No WHEN/THEN catalogues, no implementation detail the code will show.
- CLI reports state and serves instructions. Process (TDD, review gates) belongs to skills, opt-in.

## Tests

- Snapshot tests built from real `openspec/` folder shapes. Fixture text is invented: no private project content in the public repo.
- Checks: `hk check --all` (steps in `hk.pkl`), then `cargo test`. CI runs both. hk reads the git index => run it in a colocated workspace.
