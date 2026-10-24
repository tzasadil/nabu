# Blueprint nabu-mvp/5-migrate

## Feature description

`nabu migrate` prints a prompt that tells an agent how to convert a repo's spec folder to the nabu layout. The agent moves the files. nabu ships no move code => the same prompt works for other spec frameworks, and `jj op restore` undoes a bad run. OpenSpec is the worked example, because 9 local repos use it.

## Agent Decisions

- **Prompt format reuses slice 3.** Header facts plus tagged blocks from `src/prompt.rs`. Agents already parse that shape.
- **Dispatch before layers.** `migrate` needs only the root and the embedded defaults, so `main::run` dispatches it before `Layers::load` and `Config::load`. A broken `nabu/` left by a failed attempt never blocks the retry.
- **Header is only the root.** `migrate: <root>`. The agent finds the source spec folder itself and resumes when `nabu/` already exists. No framework-specific probe.
- **Default copies found by hash.** The built-in `specless` was ported from the reference repo's copy with deliberate edits, so a text comparison would need LLM judgment. The `mapping` text instead lists sha256 sums of the reference repo's original `openspec/schemas/specless/` files (`schema.yaml`, `templates/blueprint.md`, `templates/backlog.md`, `templates/tasks.md`). A matching file is deleted. Any other file is kept as a repo override and reported.
- **Prompt text lives in `src/migrate.md`**, embedded with `include_str!`. Not a layer file: the migrate prompt is built in only (one consumer).
- **`openspec-refine` maps to nothing.** `nabu-inscribe` already runs refine after its artifacts. A rule sentence whose only demand is to run refine is dropped and reported. Mapping it to `nabu-inscribe` would make propose call itself.

## User Decisions

- Migration is a prompt, not code. Command kept at the user's request. Prompt not layered.
- Changes are ported as they are, keeping their schema name, even when no layer defines that schema (legacy changes in the reference repo). Their `nabu list` rows and `nabu status` show the unknown-schema error. That is expected, not a migration failure.
- Repo schema and templates equal to the built-in defaults are deleted. Real differences stay as repo overrides and are listed in the report.
- Old skill names and `openspec/` paths in `context` and `rules` are rewritten.
- Files and folders with no mapping move into `nabu/` unchanged and are listed in the report. Nothing else is deleted.
- Final check: `nabu list` and `nabu status` on every migrated tablet.
- Port only what exists. A slice listed in `stack.md` without a source folder is skipped and reported, never created empty.

## Flow

- `nabu migrate` -> find root (slice 1 rule; a jj repo without `nabu/` resolves to its `.jj/` root) -> print header and blocks. No layer or config load.
- Agent -> follows `instruction` -> moves files -> runs `nabu list`, `nabu status` per tablet -> reports.

## Contracts

- **Header.** `migrate: <absolute root>`.
- **Blocks.** In this order: `instruction`, `layout`, `mapping`. Same block format as slice 3.
- **instruction block** tells the agent, in order:
  1. Work in one jj revision for this repo. Undo is `jj op restore`. Find the source spec folder. If `nabu/` already exists, resume.
  2. Convert the source using `layout` and `mapping`.
  3. Delete repo schema and template files in `openspec/schemas/specless/` whose sha256 matches the list in `mapping`. Keep every other file as a repo override, a matching file in another schema folder included (reported: the built-in fallback exists only for `specless`).
  4. Apply the name and path map to every text file kept under `nabu/config.toml` and `nabu/schemas/`. Report OpenSpec-only text left in kept overrides (`.openspec.yaml`, `skip_specs`) without rewriting it.
  5. Place unmapped items per the unmapped rule in `mapping`. Delete nothing else. Remove the emptied source folder.
  6. Run `nabu list`, then `nabu status <tablet>` for every listed tablet. Fix every error except an unknown schema carried over from the source.
  7. Report: kept overrides, dropped refine sentences, unmapped items and where they went, unknown schemas with counts, unknown skill names, skipped stack slices, stale names left in stack files, anything not converted.
- **layout block** states the nabu layout and formats: `nabu/config.toml`, `nabu/schemas/<name>/schema.toml` + `templates/`, `nabu/tablets/<tablet>/.nabu.toml` (`schema = "<name>"`), stacked tablets with `stack.md` and `<N>-<slice>/.nabu.toml` (empty marker), `nabu/archive/YYYY-MM-DD-<tablet>/`. Names are kebab-case; slice names start with a letter. TOML keys as in slice 1 `File keys`.
- **mapping block** gives the OpenSpec example:
  - `openspec/config.yaml` -> `nabu/config.toml` (`schema`, `context`, `rules`). A `#` line inside a block scalar is text and stays. Only `#` lines outside block scalars are comments.
  - `openspec/schemas/<name>/schema.yaml` -> `nabu/schemas/<name>/schema.toml` (drop `version`; add `stack = "stack.md"` when the repo has stacks), `templates/` copied.
  - `openspec/changes/<name>/` -> `nabu/tablets/<name>/`. An active change is excluded only when an active stack names it. An archived stack consumes only `changes/archive/<date>-<change>/`, so a same-named active folder stays a plain tablet (the reference repo has such duplicates). `.openspec.yaml` -> `.nabu.toml` keeping only `schema`, unchanged even when no layer defines it.
  - `openspec/stacks/<name>/stack.md` plus its slice changes -> one stacked tablet `nabu/tablets/<name>/` with `stack.md` and slices `<N>-<slug>/`. Each slice heading `### <N>. <title>` gives N and the slug. Integer N is used as is. Otherwise (`3.5`) the next free N is used and reported. Slug = the title only: lowercase, runs of non-alphanumerics to `-`, trimmed, prefixed `s-` if it starts with a digit. The slice's source folder is the one its `**Change:**` line names, else `changes/<name>-<N>/`, else `changes/<name>-<N>-*/` (hyphen after N required). An archived stack without `**Change:**` lines uses its folder name without the date prefix as `<name>`, searched as `changes/archive/<date>-<name>-<N>[-*]/` with any date. A listed slice with no source folder is skipped and reported. Every slice folder gets the empty `.nabu.toml` marker. The tablet's `.nabu.toml` schema comes from the first slice's `.openspec.yaml`, else the config `schema`.
  - `openspec/changes/archive/<date>-<name>/` -> `nabu/archive/<date>-<name>/`. Archived stacks and their archived slices combine the same way under the stack's archive date. An undated stack archive takes the latest date of its slices and is reported.
  - Stack files keep their text. Old change names inside them are reported, not rewritten.
  - **Name and path map.** `openspec-<x>` -> `nabu-<x>` for `apply`, `archive`, `learn`, `outline`, `propose`, `tweak`, `verify`. Refine, named as skill `openspec-refine` or as file `openspec-propose/refine.md`: drop sentences whose only demand is to run it, report them. `openspec/changes/archive/` -> `nabu/archive/`. `openspec/changes/<name>` -> `nabu/tablets/<name>`. `openspec/stacks/...` mentions are reported, not rewritten. `openspec/` -> `nabu/` last. Other `openspec-*` names are reported, not rewritten.
  - **Default hashes.** sha256 of the reference repo's original `schema.yaml` and three templates, listed with their paths.
  - **Unmapped rule.** A top-level unmapped item keeps its relative path under `nabu/` (`openspec/specs/` -> `nabu/specs/`, `openspec/KNOWN-ISSUES.md` -> `nabu/KNOWN-ISSUES.md`). An unmapped item inside a mapped folder moves into that folder's target (a change's `specs/`, `stacks/<name>/progress.md` -> the stacked tablet). Never create a non-tablet folder directly under `nabu/tablets/`.
- **Exit.** 0 with the prompt on stdout, even when `nabu/` holds a broken config. Slice 1 root errors apply unchanged.

## Out of scope

- Code that moves files.
- Running the migration on existing repos (slice 9).
- Layering the prompt.
- Defining legacy schemas. Their tablets stay with an unknown schema until the user acts.

## Gotchas

- The default hashes match only byte-identical copies of the reference repo's originals. Most repos hold a byte-identical `schema.yaml` (checked during the outline). Edited copies stay as overrides by design.
- One repo has the same change counts as the reference repo. Check in slice 9 whether it is a second checkout before migrating both.
