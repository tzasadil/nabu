# Stack: nabu-mvp

## Overview

`nabu` replaces the OpenSpec CLI and the 7 `openspec-*` skills for one user. The CLI reads and writes planning folders under `nabu/`, merges config and templates from three layers, and serves prompt text to agents. The skills and their agents move into this repo, call `nabu`, and hold no copies of nabu templates. `nabu init` installs them globally or into one project. The 9 existing `openspec/` repos move over through a migration prompt that an agent runs.

## Source requirements

- jj native: skills use jj (autopropagation, pin -> diff, `jj op restore` as undo) -> slices 6, 7. CLI never calls jj or git -> Decisions.
- Stacks first class: a tablet with `stack.md` holds one subfolder per slice -> slices 1, 2, 3, 4.
- Fully configurable, global settings and templates -> slices 1, 3.
- Templates in one place, never copied into skills -> slices 3, 6.
- Self-learning through `nabu-learn` -> slice 7.
- Backlog as an extra -> slice 1 (schema), slice 7 (`nabu-archive` settles it).
- Folder layout `nabu/tablets/`, `nabu/archive/` -> slices 1, 4.
- MVP commands `new`, `status`, `list`, `instructions`, `archive` -> slices 1 to 4.
- `nabu migrate` prints a prompt an agent follows -> slice 5. Running it on existing repos -> out of scope.
- Skills live in this repo -> slices 6, 7. `nabu init` (global, default) and `nabu init --local` -> slice 8.
- Architecture rules stay with the skills -> slice 6.
- `nabu-learn` scoped to nabu, with rule-writing guidance from `writing-skills` -> slice 7.
- Upstream #1084 (done by content) -> slice 2. #1831 (ready to archive) -> slice 2. #1907 (deferred on archive) -> slice 7, see Decisions.
- `validate` dropped -> Decisions.

## Existing Foundation

- This repo: empty `cargo init` crate (edition 2024). Nothing to reuse in code.
- Formats to carry over, from `<reference repo>/openspec/`:
  - `schemas/specless/schema.yaml`: artifacts with `id`, `generates`, `template`, `requires`, `instruction`, plus an `apply` block (`requires`, `tracks`, `instruction`). Most repos hold a byte-identical copy. The variant repo differs.
  - `schemas/specless/templates/`: `blueprint.md`, `tasks.md`, `backlog.md`.
  - `config.yaml`: `schema`, `context`, per-artifact `rules`. Its `context` names `openspec-*` skills and `openspec/changes/` paths.
  - `.openspec.yaml` per change: `schema`, `created`, `skip_specs`.
  - Stack today: `stacks/<name>/stack.md` apart from slice changes `changes/<name>-<N>/`.
  - Data with no mapping: `specs/`, `KNOWN-ISSUES.md`, extra files such as `stacks/verification-agent/progress.md`, mixed dated and undated `stacks/archive/` folders.
- Skills and agents in `/data/src/wiki/`:
  - `skills/openspec-{apply,archive,learn,outline,propose,tweak,verify}/`.
  - `stack.md` template in `skills/openspec-outline/templates/`. Architecture rules in `skills/openspec-propose/architecture-rules.md`, read by outline, propose reviewers, and learn.
  - Agents `agents/openspec-opponent.md`, `agents/openspec-reviewer.md`, `agents/artifacts.md`, `agents/implementer.md`. `~/.claude/agents/` holds real file copies. `implementer.md` there differs from the wiki copy.
  - `skills/with-implementer/` (with `implementer-rules.md`): used by apply, tweak, verify. learn edits `implementer-rules.md`.
  - Optional dependencies, staying in the wiki: `debugging-duck` agent (learn mines its findings), `renaming` skill (verify), `rule-forge` skill (learn sends coding-standard candidates).
  - `skills/openspec-propose/refine.md` runs `openspec validate`. `skills/openspec-propose/scripts/check_bundle.py` checks the bundle.
  - `skills/openspec-learn/scripts/mine_critiques.py` matches `openspec-(\w+)` and `openspec/changes/`.
  - `skills/writing-skills/SKILL.md`: style and steering rules learn will copy.
- Skills call: `list --json`, `status --change --json`, `instructions <artifact> --change --json`, `instructions build --change --json`, `new change`. `openspec-archive` moves stacks with `mv`. No skill script parses CLI output.
- Deploy today: `/data/src/wiki/justfile` (`just link`) puts one symlink per skill in `~/.agents/skills/`. `~/.claude/skills` resolves to the same folder. One repo already uses a project `.claude/skills -> ../.agents/skills` link.
- Upstream agent contract: `docs/agent-contract.md` in Fission-AI/OpenSpec. Output samples: `/tmp/nabu-{status,instr,apply,list}.json`.

## Slices

### 1. config-and-new
**Depends on:** nothing in this stack
**Delivers:**
- `nabu new <tablet>` creates `nabu/tablets/<tablet>/` with `.nabu.toml` naming its schema.
- `nabu new <tablet> --stack` also writes the schema's stack template.
- `nabu new <tablet>/<slice>` creates `<N>-<slice>/` with an empty `.nabu.toml` marker in a stacked tablet, N = next free number. jj tracks files only => the marker keeps the slice alive. A slice = `<N>-<slice>/` holding the marker.
**Must include:**
- Three layers: built-in defaults (embedded in the binary) -> `~/.config/nabu/` -> repo `nabu/`. Schema and each template: nearest layer wins the whole file. `context` and per-artifact `rules`: appended, global first.
- Built-in schema `specless`: artifacts `blueprint`, `backlog`, `tasks`, plus a `stack = "stack.md"` key naming the stack template. `tasks` does not require `backlog`.
- TOML for `config.toml`, `schema.toml`, `.nabu.toml`. Prose inline.
- Root: nearest ancestor holding `nabu/config.toml` or `nabu/tablets/`, else nearest `.jj/` root.
- Tablet and slice lookup. A slice resolves by `<tablet>/<N>`, `<tablet>/<slice>`, or `<tablet>/<N>-<slice>`. Slice order comes from N. Slices inherit the tablet's schema. Unknown tablet or slice -> error listing what exists.
- Malformed config or schema -> non-zero exit naming the file and the parse error.
- Fixture set, hand-converted from the reference repo: one plain tablet, one stacked tablet with slices, one archived tablet, the variant repo's schema.
**Excludes:**
- Writing artifacts. Skills write them.

### 2. status-and-list
**Depends on:** slice 1 layers, root, lookup
**Delivers:**
- `nabu status <tablet>` shows each artifact as done, ready, or blocked, plus task progress. On a stacked tablet it shows the same per slice. `nabu status <tablet>/<slice>` shows one slice under the tablet header.
- `nabu list` shows active tablets with progress and a "ready to archive" mark.
**Must include:**
- Schema graph checks: duplicate ids, unknown `requires` and `build.requires`, `build.tracks` generated by an artifact, artifact templates resolvable, no cycles. Error names the schema file.
- Done = the file exists, is not empty, and differs from its template (#1084).
- Progress counts checkboxes only in the schema's `build.tracks` file.
- Ready to archive = every tracked task checked, in every slice (#1831).
- One compact text format for humans and agents.
**Excludes:**
- Prompt text. That is slice 3.

### 3. instructions
**Depends on:** slice 1 layers and lookup, slice 2 artifact state
**Delivers:**
- `nabu instructions <artifact> <target>` prints the artifact's template, instruction, merged context and rules, output path, and dependency state.
- `nabu instructions build <target>` prints the build instruction, context files, and pending tasks.
- `nabu instructions stack <tablet>` prints the stack template and context.
- `<target>` is a tablet or `<tablet>/<slice>`.
**Must include:**
- Output is the prompt text itself, ready to feed to a model.
- Every template and rule comes from the layers.
- A `[rules]` entry naming an artifact the tablet's schema lacks -> error naming the config file, the id, and the schema.
- `generates` must be a plain file name (no `/`, `\`, leading `.`), checked in `schema::resolve`, before it is printed as a path skills write to.
**Excludes:**
- Workflow policy (review gates, TDD). It belongs to skills or config.

### 4. archive
**Depends on:** slice 1 lookup
**Delivers:**
- `nabu archive <tablet>` moves the whole tablet to `nabu/archive/YYYY-MM-DD-<tablet>/`.
**Must include:**
- Stacked tablets archive as one folder, `stack.md` and all slices included.
- Target folder exists -> refuse, nothing moved.
**Excludes:**
- Backlog handling. `nabu-archive` settles the backlog before it calls `nabu archive`.

### 5. migrate
**Depends on:** slice 1 layout and formats, slice 2 `list` and `status` as checks
**Delivers:**
- `nabu migrate` prints a prompt that tells an agent how to convert a repo's spec folder to `nabu/`.
**Must include:**
- The prompt states the nabu layout, file formats, and a final check with `nabu list` and `nabu status` on every migrated tablet.
- OpenSpec mapping as the worked example: `changes/` -> `tablets/`, `stacks/<name>/` plus `changes/<name>-<N>/` -> one stacked tablet, `changes/archive/` -> `archive/`, YAML -> TOML, `.openspec.yaml` -> `.nabu.toml`.
- Repo schema and templates equal to the built-in defaults are deleted. Real differences stay as repo overrides and are listed in the report.
- Old skill names and `openspec/` paths in `context` and `rules` are rewritten.
- Files and folders with no mapping move into `nabu/` unchanged and are listed in the report. Nothing is deleted except the default copies above.
- Prompt text is built in only, not layered.
**Excludes:**
- Code that moves files. The agent moves them. `jj op restore` undoes a bad run.

### 6. planning-skills
**Depends on:** slice 1 `new`, slice 2 `status` and `list` output, slice 3 `instructions`
**Delivers:**
- `skills/nabu-outline`, `skills/nabu-inscribe`, `skills/nabu-tweak` in this repo, working on tablets through `nabu`.
- Agents `agents/nabu-opponent.md` and `agents/nabu-artifacts.md`.
- `nabu-outline` creates a stacked tablet with `nabu new --stack` and scaffolds every slice with `nabu new <tablet>/<slice>`, so `status` and `list` see all planned slices. `nabu-inscribe` fills slices or plain tablets.
- This repo links `.agents/skills -> ../skills`, `.claude/skills -> ../.agents/skills`, and `.claude/agents -> ../agents`, so the skills and agents run here before slice 8.
**Must include:**
- No nabu template text in skills. `stack.md` comes from `nabu instructions stack`.
- `learned-rules.md` stays in `skills/nabu-inscribe/` with its headings and IDs.
- `check_bundle.py` replaces `openspec validate` in refine.
- jj patterns kept: pin -> diff at subagent boundaries, one revision per slice.
- Slice markers (`<N>-<slice>/.nabu.toml`) live in the stack revision with `stack.md`. Each slice's artifacts live in its own revision.
**Excludes:**
- Slimming skills or making them more generic. Next stack.

### 7. execution-skills
**Depends on:** slice 4 `archive`, slice 6 skill and agent layout
**Delivers:**
- `skills/nabu-build`, `skills/nabu-verify`, `skills/nabu-archive`, `skills/nabu-learn` in this repo, calling `nabu`.
- `skills/with-implementer/` and `agents/implementer.md`, keeping their names. Agent `agents/nabu-reviewer.md`.
- `nabu-archive` settles each backlog item (keep, move to a new tablet, drop), then calls `nabu archive`.
- `nabu-learn` mines only sessions that used `nabu` or `nabu-*` skills and edits only `skills/` in this repo.
- Built-in `defaults/` text (`tasks.md` template, `apply` instruction) names `nabu-verify` instead of `openspec-verify`.
- `nabu-inscribe` refine hands off to `/nabu-build` (slice 6 stops at "ready" until this slice lands).
**Must include:**
- `skills/nabu-learn/rule-writing.md`, copied from `writing-skills`: writing style, no speculative guardrails, positive targets, pruning, sprinkling, restated globals. Linked at the step where learn drafts a rule.
- `mine_critiques.py` matches `nabu-*` skills and `nabu/` paths.
- `implementer.md` source picked between the wiki copy and `~/.claude/agents/` before the move.
- `debugging-duck`, `renaming`, `rule-forge` used only when installed. Their absence never blocks a skill.
**Excludes:**
- Slimming `nabu-verify`. Next stack.
- Removing the old skills. Later cutover tablet (Out of scope).

### 8. init
**Depends on:** slices 6, 7 skills and agents
**Delivers:**
- `nabu init` (same as `--global`) copies the embedded skills and agents into `~/.agents/skills/` and the global agent folders, and writes a starter `~/.config/nabu/config.toml`.
- `nabu init --local` copies them into the project's `.agents/skills/`, links `.claude/skills` to them, and writes `nabu/config.toml`. Nothing is written outside the repo.
**Must include:**
- Skills and agents embedded in the binary => installed copies match the CLI.
- Running again overwrites. No merge logic.
- Starter configs only, no template copies.
- Local projects still read the global layer.
**Excludes:**
- Version checks of installed copies. See Out of scope.

### 9. global-init
**Depends on:** slice 8
**Delivers:**
- `nabu init` run once on this machine: nabu skills in `~/.agents/skills/`, agents in `~/.claude/agents/`, starter `~/.config/nabu/config.toml` when missing.
**Must include:**
- `cargo install --path .` first, so the binary carries the current skills.
- Failure mode and rollback stated before the run. The user confirms the run.
- The old `openspec-*` skills and agents stay installed beside the nabu ones.
**Excludes:**
- Migrating any repo, editing the wiki, removing old skills, updating wiki consumers, merging learn state. User decision: nothing outside this repo except the global install.

## Decisions

- Term: tablet = one unit of planned work (upstream: change). "Change" means a jj revision only.
- jj native lives in the skills. The CLI never calls jj or git and has no `--dry-run`. `jj op restore` is the undo.
- One text output format. No `--json`. Upstream's `--json` serves non-interactive scripting and multi-root stores, which nabu does not have.
- Built-in layer kept: zero-config start, hermetic tests. Global and repo layers override whole files.
- Whole-tablet archive only.
- `.nabu.toml` per tablet holds the schema. No `created` field.
- TOML with inline prose. `serde_yaml` is deprecated, and TOML is the Rust norm.
- Migration is a prompt, not code => works for other spec frameworks too. Command kept at the user's request. Prompt not layered (one consumer).
- Architecture rules stay with the skills. They are skill content, not nabu config.
- #1907 moves from the CLI to `nabu-archive`: printing the backlog during the move leaves no chance to act on it.
- Same skill name local and global: Claude Code runs the personal (global) copy. Codex reportedly prefers the project copy (unverified). Both come from one binary => identical when refreshed together.
- nabu ships its hard dependencies (`with-implementer`, `implementer`, opponent, reviewer, artifacts agents). General tools (`debugging-duck`, `renaming`, `rule-forge`) stay in the wiki as optional.
- `validate` dropped: the `specless` schema has no spec deltas.

## Cross-cutting Concerns

- Errors state the cause and a fix the user can paste and run.
- nabu never prompts interactively.
- No artifact names hard-coded in command logic. Roles (`stack`, `build.tracks`) come from the schema.
- Snapshot tests on the slice 1 fixture set. Later slices extend it.
- Text output of `status`, `list`, and `instructions` is the contract slices 6 and 7 consume.
- From slice 1: printed paths are root-relative (`nabu/tablets/...`), global-layer paths absolute. Errors print `error: ...` then `fix: ...` to stderr, exit 1.
- From slice 1: config is parsed once in `main::run` before dispatch. Layers load only `config.toml`, `schemas/<name>/schema.toml`, `schemas/<name>/templates/*.md`. Any new layer file must be added to `read_layer` in `src/layers.rs`.
- From slice 8: `nabu init` installs the skills and agents embedded at build time. Rerun `cargo install --path .` before `nabu init` so the binary carries the current skills. Global init replaces same-named entries in `~/.agents/skills` (the wiki `with-implementer` symlink) and refuses symlinked leaf folders.
- From slice 7: all 8 skills live in `skills/` and run in this repo. `with-implementer` also exists globally (wiki): slice 9's global install replaces that link with the nabu copy. Planning from slice 8 on dogfoods `nabu-inscribe`.
- From slice 6: skills call a bare `nabu` installed by `cargo install --path .`. Rerun it after any CLI or `defaults/` change (slices 7, 8 gates).
- From slice 5: `nabu migrate` runs before layers and config load. Untested prompt cases for the later cutover to watch: refine requested by path (`openspec-propose/refine.md`), archived stacks without `**Change:**` lines (2 repos), a hash match outside `specless`. The repo with the reference repo's change counts is a separate jj repo, not a second checkout (checked in slice 9 planning).
- From slice 4: `tablets::ARCHIVE_DIR` names `nabu/archive`. Archive needs no schema resolve.
- From slice 3: prompts are built by `src/prompt.rs` (header lines + tagged blocks, LF endings) and `src/instructions.rs`. `report::stack_done` owns the stack file's done state. Reuse `prompt` for any new prompt-like output (slice 5 `migrate`).
- From slice 2: reuse these owners. `tablets::split_target` (target parsing), `tablets::stack_file` / `require_stack_file` (stacked or plain), `report::load` + `report::read` (artifact states, progress, ready to archive), `layers::read_file` (not found -> none, else error), `table::render` with `Row::Tail` for unbounded text. Graph checks run inside `schema::resolve`; `ResolvedSchema.templates` holds artifact template text. Commands return `Outcome { text, failed }`.
- From slice 1: schema fields and slice lookup used by later slices carry `#[allow(dead_code)]` naming the consuming slice. Drop each allow when the caller lands.

## Out of scope

- Slimming `nabu-verify` and making skills more generic (next stack).
- `status` returning a jj pin (trigger: skills pin right after `status` in most runs).
- `--json` output (trigger: a script, not an LLM, needs to parse output).
- Version stamp in installed skills with a stale-copy warning (trigger: first confusion from a stale local copy).
- Archive guard for unchecked tasks (#2006). `depends_on` between tablets (#1915). Drafts (#1818).
- Cutover beyond the global install (later tablet): migrate existing `openspec/` repos with `nabu migrate`, merge `openspec-learn` rules and receipts written after slice 7, remove the old `openspec-*` skills and agents from the wiki and `~/.agents`, update the skills and docs that still name `openspec-*`.
