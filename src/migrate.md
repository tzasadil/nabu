@@ instruction
Convert this repo's spec folder to the nabu layout. You move the files. nabu has no move code. Use `layout` for the target and `mapping` for the source. `mapping` describes OpenSpec. For another spec framework, apply the same ideas.

Do these steps in order.

1. Work in one jj revision for this repo. Run `jj new -m "migrate specs to nabu"` unless you are resuming. To undo the whole run, use `jj op restore` to the operation before you started (find it with `jj op log`). Use `mv` and plain file edits. Never use git. Find the source spec folder (for OpenSpec, `openspec/`). If `nabu/` already exists, an earlier run was cut short: list what is done, then continue. Do not redo or overwrite finished work.
2. Convert the source with `layout` and `mapping`. Move files. Do not retype their content. Edit text only where this prompt says so.
3. Compute `sha256sum` of every file in the source schema folders. Delete each file inside `openspec/schemas/specless/` whose sum is in the default hashes in `mapping`: it is a copy of a built-in default. Keep every other file as a repo override. A file with a matching sum in any other schema folder is kept too: report it, because the built-in fallback exists only for `specless`. Convert a kept `schema.yaml` to `schema.toml`. Copy a kept template (step 4 then rewrites names in it). Drop `version`. If the repo has stacks, add `stack = "stack.md"` to a kept schema named `specless`. A kept schema with another name also needs its own `templates/stack.md`: write `# Stack: <name>` and an `## Overview` heading, and report it.
4. Apply the name and path map from `mapping` to every text file you keep or write under `nabu/config.toml` and `nabu/schemas/`. Rewrite nothing else. In kept overrides, report OpenSpec-only text you did not rewrite (`.openspec.yaml`, `skip_specs`, `openspec validate`, and similar).
5. Move items with no mapping per the unmapped rule in `mapping`. Delete nothing else. When the source spec folder is empty, remove it.
6. Run `nabu list`. Then run `nabu status <tablet>` for every listed tablet. `nabu list` also prints a row with an error for each tablet it cannot read. Fix every error, except an unknown schema that came from the source: those tablets keep their schema name on purpose. Archived tablets are not listed: check by eye that every folder in `nabu/archive/` is named `YYYY-MM-DD-<tablet>`.
7. Write a report. List: kept overrides, dropped refine sentences, unmapped items and where each went, unknown schemas with the number of tablets per schema, unknown `openspec-*` names, skipped stack slices, renumbered slices (`3.5`), undated stack archives and the date you gave them, old change names left in stack files, and anything you did not convert.

Stop and report when a target path already exists and is not yours, or a name cannot be made valid by the rules in `layout`.

@@ layout
Target layout, relative to the repo root:

- `nabu/config.toml`: repo config.
- `nabu/schemas/<name>/schema.toml` and `nabu/schemas/<name>/templates/*.md`: repo schema overrides.
- `nabu/tablets/<tablet>/`: one unit of planned work (OpenSpec: a change). It holds `.nabu.toml`, `blueprint.md`, `backlog.md`, `tasks.md`, and any other files.
- `nabu/archive/YYYY-MM-DD-<tablet>/`: a finished tablet, moved whole.

A stacked tablet has a `stack.md` next to `.nabu.toml`. Each slice is a subfolder `<N>-<slice>/` that holds an empty `.nabu.toml` file and the slice's artifact files. A slice folder without that marker file is not a slice.

Names: lowercase letters, digits, and hyphens only. A tablet name starts with a letter or a digit. A slice name starts with a letter. `N` is a whole number.

File formats. All TOML keys not listed here are errors.

- `config.toml`, all keys optional:
  - `schema = "<name>"`.
  - `context = '''...'''`: text. Use a multi-line literal string `'''`. If the text holds `'''`, use `"""` and escape `\` and `"`.
  - `[rules]` table: `<artifact id> = ["rule text", ...]`. Use one literal string `'...'` per rule. If a rule holds `'`, use `"..."` and escape `\` and `"`.
- `schema.toml`:
  - Required: `name`, and one `[[artifacts]]` table per artifact with `id`, `generates`, `template`.
  - Optional per artifact: `description`, `instruction`, `requires` (list of ids).
  - Optional top level: `description`, `stack` (template file name, for example `stack = "stack.md"`).
  - Required `[build]` table with `requires` (list of ids) and `tracks` (a generated file name). Optional `build.instruction`.
  - Source YAML folded text (`>`) joins its lines with one space, and a blank line is a line break. Write the result as a multi-line TOML string.
- `.nabu.toml` of a tablet: `schema = "<name>"`. Slice markers are empty files.

@@ mapping
OpenSpec to nabu. Paths are relative to the repo root.

- `openspec/config.yaml` -> `nabu/config.toml`. Keep `schema`, `context`, `rules`. A `#` line outside a YAML block scalar (`|` or `>`) is a comment: drop it or turn it into a TOML comment. A `#` line inside a block scalar is text: keep it. Unindent the `context` block text.
- `openspec/schemas/<name>/schema.yaml` -> `nabu/schemas/<name>/schema.toml`. Drop `version`. The YAML `apply` block becomes the `[build]` table. `openspec/schemas/<name>/templates/` -> `nabu/schemas/<name>/templates/`, files as they are. A deleted `schema.toml` or template falls back to the built-in `specless` default. That works only for the schema named `specless`.
- `openspec/changes/<name>/` -> `nabu/tablets/<name>/`, for every folder in `openspec/changes/` except `archive` and except slice changes that an active stack names (see below). Only an active stack consumes an active change: an archived stack consumes only `openspec/changes/archive/<date>-<change>/`, so a same-named active folder stays a plain tablet. Inside, `.openspec.yaml` -> `.nabu.toml` with only the `schema` value, kept even when no built-in or repo schema has that name. Drop `created` and `skip_specs`. If `.openspec.yaml` is missing, use the `schema` of the config and report it.
- Stacks: `openspec/stacks/<name>/stack.md` plus its slice changes -> one stacked tablet `nabu/tablets/<name>/`.
  - `stack.md` keeps its text.
  - Each stack heading `### <N>. <title>` is one slice. Whole `N` is used as it is. For any other `N` (for example `3.5`), use the next free whole number and report it.
  - Slice folder name = `<N>-<slug>`. The slug comes from the title only: lowercase, each run of non-alphanumeric characters becomes `-`, trim `-` at both ends, prefix `s-` if it starts with a digit.
  - The slice source is the change named in the `**Change:**` line below the heading. Take the name inside the backticks. If there is no such line, use `openspec/changes/<name>-<N>`, else a folder named `<name>-<N>-` followed by more text (the hyphen after `N` is required, so `<name>-1` never matches `<name>-10`). A listed slice with no source folder is skipped and reported. Never create an empty slice.
  - Move the slice source's files into the slice folder. Delete its `.openspec.yaml` (the marker replaces it). Put an empty `.nabu.toml` in the slice folder.
  - The tablet's `.nabu.toml` takes its `schema` from the `.openspec.yaml` of the first slice that has a source. Otherwise it takes the config `schema`. Report slices whose schema differs from that.
  - Other files in `openspec/stacks/<name>/` (for example `progress.md`) move into the stacked tablet folder. A `openspec/stacks/<name>/` folder without `stack.md` is not a stack: treat it as an unmapped top-level item.
- Archive: `openspec/changes/archive/<date>-<name>/` -> `nabu/archive/<date>-<name>/`.
  - An archived stack (`openspec/stacks/archive/<date>-<name>/` or undated `openspec/stacks/archive/<name>/`) combines with its slices the same way as an active stack. Slice sources are `openspec/changes/archive/<date>-<change>/`, with the change from the `**Change:**` line and any date. Without a `**Change:**` line, `<name>` is the stack folder name without its date prefix: use `openspec/changes/archive/<date>-<name>-<N>`, else `<date>-<name>-<N>-` followed by more text, with any date. The result is `nabu/archive/<date>-<name>/` with `stack.md`, `.nabu.toml`, and slice folders with their markers, built like an active stacked tablet.
  - For an undated stack archive, use the latest date among its slice sources. Report the date you gave.
  - Archived changes that belong to no stack move as plain folders. They keep their files, including `.openspec.yaml`.
- Name and path map. Apply the rules from the top.
  - Apply the more specific path rules before `openspec/` -> `nabu/`.
  - `openspec-apply` -> `nabu-build`. `openspec-propose` -> `nabu-inscribe`.
  - `openspec-<x>` -> `nabu-<x>` for `x` in `archive`, `learn`, `outline`, `tweak`, `verify`.
  - `openspec-refine` has no nabu skill, because `nabu-inscribe` already runs the check after it writes the artifacts. Drop each sentence or list item whose only demand is to run refine, named either as skill `openspec-refine` or as the file `openspec-propose/refine.md` ("run the refine procedure"). Report every dropped text. Report other mentions of either name without rewriting them.
  - `openspec/changes/archive/` -> `nabu/archive/`.
  - `openspec/changes/<name>` -> `nabu/tablets/<name>`.
  - `openspec/stacks/...` has no fixed target: report each mention, do not rewrite it.
  - `openspec/` -> `nabu/`.
  - Any other `openspec-*` name is reported, not rewritten.
- Stack files keep their text. Report old change names that remain inside them.
- Default hashes. These are sha256 sums of the built-in defaults as OpenSpec stored them.

```
9067f0b770c3b14a74a71b383745559058466a2e12651e80729130060b786792  openspec/schemas/specless/schema.yaml
10120b26262abe0c2d856c5c42b2aea595593549ac3835ed669469e6d90a183a  openspec/schemas/specless/templates/blueprint.md
a70eec8b82cb7e2f70849269eecc31ce0a4fbfcc412957a8f7695ad4996a69f0  openspec/schemas/specless/templates/backlog.md
6fc80aa569942d680cd6899739d4c9fa0ad27fb2385bff1e2092fb9a6e101900  openspec/schemas/specless/templates/tasks.md
```

- Unmapped rule.
  - A top-level item in the source folder with no mapping keeps its relative path under `nabu/`: `openspec/specs/` -> `nabu/specs/`, `openspec/KNOWN-ISSUES.md` -> `nabu/KNOWN-ISSUES.md`.
  - An unmapped item inside a mapped folder moves into that folder's target: a change's `specs/` stays in its tablet, and `openspec/stacks/<name>/progress.md` goes to the stacked tablet.
  - Never create a folder under `nabu/tablets/` that is not a tablet.
