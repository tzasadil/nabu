# Blueprint nabu-mvp/1-config-and-new

## Feature description

First working `nabu` binary. It finds the planning root, loads config and schemas from three layers, and creates tablets and slices:

- `nabu new <tablet>`: creates `nabu/tablets/<tablet>/` with `.nabu.toml`.
- `nabu new <tablet> --stack`: also writes the schema's stack template as `stack.md`.
- `nabu new <tablet>/<slice>`: creates `<N>-<slice>/` inside a stacked tablet, N = next free number.

The slice also owns tablet and slice lookup, the built-in `specless` schema, and the test fixture set later slices extend.

## Agent Decisions

- **Layer folders.** Each layer has the same shape: `config.toml` plus `schemas/<name>/schema.toml` and `schemas/<name>/templates/*.md`. Built-in layer = `defaults/` in this repo, embedded at compile time. Global layer = `$XDG_CONFIG_HOME/nabu/`, else `~/.config/nabu/`. Repo layer = `<root>/nabu/`. Same shape => one loader for all three.
- **Global path without a crate.** `dirs` and similar crates return `~/Library/Application Support` on macOS. The user wants `~/.config/nabu/`, so the path comes from `XDG_CONFIG_HOME` / `HOME` directly. Tests set `XDG_CONFIG_HOME` and `HOME` to temp folders => no extra env var.
- **Template lookup per file.** A template resolves as `schemas/<schema>/templates/<file>` in the nearest layer that has that file. A repo can override one template of the built-in schema without copying `schema.toml`.
- **Strict parsing.** Unknown keys in any TOML file are errors => typos fail loudly instead of being ignored.
- **Schema checks in this slice** (see `Schema validity`): only what `new` uses. Graph checks (duplicate ids, unknown `requires`, `build.tracks`, artifact templates, cycles) live in slice 2, where the graph is first read.
- **Name rule.** Tablet and slice names are kebab-case: lowercase ASCII letters, digits, hyphens. Tablets start with a letter or digit, which matches every existing change name (`42-empty-list-notice`). Slices start with a letter => a slice name never looks like `<N>` or `<N>-<name>`, so lookup is never ambiguous.
- **Embedding.** The built-in layer is an explicit `include_str!` list over `defaults/`. No embedding crate. A test fails when a file in `defaults/` is missing from the list. Global and repo layers are read up front into the same in-memory map of path -> text the embedded list provides. One loader works on three maps, no interface. Map keys are the paths => error messages name files.
- **Printed paths.** Paths inside the root print relative to the root. Global-layer paths print absolute. Stable for snapshots and for later slices that read the output.
- **Built-in text uses "tablet".** Templates and instructions copied from the reference schema say "change". They are reworded to "tablet" (AGENTS.md term rule). No other wording changes.
- **Dependencies.** `clap` (derive), `serde`, `toml`. Dev: `insta`, `tempfile`. Integration tests run the real binary through `CARGO_BIN_EXE_nabu` => no `assert_cmd`.

## User Decisions

- Three layers, built-in -> global -> repo. Schema file and each template: nearest wins. `context` and per-artifact `rules`: appended, global first. Built-in layer exists for zero-config start and hermetic tests.
- TOML everywhere, prose inline as multi-line strings.
- `.nabu.toml` per tablet holds `schema` only. Each slice holds an empty `.nabu.toml` marker and inherits the tablet's schema. The marker keeps a fresh slice tracked by jj (jj tracks files, not folders).
- Root marker: `nabu/config.toml` or `nabu/tablets/`. Only when no ancestor has one: nearest `.jj/` root.
- Slices auto-numbered. Lookup accepts `<tablet>/<N>`, `<tablet>/<slice>`, `<tablet>/<N>-<slice>`. Order comes from N.
- Built-in schema `specless`: `tasks` does not require `backlog`. Schema key `stack` names the stack template.
- One compact text output. No `--json`. nabu never prompts.
- CLI never calls jj or git. `jj op restore` is the undo.

## Flow

- `nabu <command>` -> find root (walk up from cwd) -> load layers (built-in, global, repo) -> merge config -> resolve schema -> run command -> print result or error.
- `new <tablet>` -> check name -> tablet absent? -> schema from config `schema` key -> schema loads? -> create folder + `.nabu.toml` (+ `stack.md` with `--stack`).
- `new <tablet>/<slice>` -> tablet exists and is stacked? -> slice name free? -> N = highest N + 1 -> create `<N>-<slice>/`.

## Contracts

- **Root.** Walking up from cwd, the nearest folder holding `nabu/config.toml` or `nabu/tablets/` is the root. No such folder -> the nearest folder holding `.jj/`. Neither -> error: no nabu root, fix: run inside a jj repo or create `nabu/config.toml`.
- **Global layer path.** `$XDG_CONFIG_HOME/nabu/` when the variable is set and not empty, else `$HOME/.config/nabu/`. Neither variable set -> global layer skipped.
- **Layers.** A missing global or repo layer is skipped. A present but malformed file -> exit 1, message names the file path and the parse error. Only `config.toml`, `schemas/<name>/schema.toml` and `schemas/<name>/templates/*.md` are read. Other files (`.DS_Store`, swap files, hidden files) are ignored, even when they are not UTF-8.
- **Config merge.** `schema`: nearest layer that sets it wins. `context`: every layer's text, global before repo. `rules.<artifact>`: lists concatenated, global first.
- **Schema resolution.** `schema.toml` from the nearest layer that has `schemas/<name>/schema.toml`. Unknown schema name -> error listing the schemas found in all layers.
- **File keys.** Any other key is an error.
  - `config.toml`: `schema` (optional, schema name), `context` (optional, text), `[rules]` (optional, artifact id -> list of text).
  - `schema.toml`: `name` (required), `description` (optional), `stack` (optional, template file), `[[artifacts]]` with `id`, `generates`, `template` (required) and `description`, `instruction`, `requires` (optional, default empty), `[build]` with `requires`, `tracks` (required) and `instruction` (optional).
  - `.nabu.toml`: `schema` (required).
- **Schema validity.** Missing required key or unknown key -> parse error naming the schema file. `stack` set but its template resolves in no layer -> error naming the schema file and the template. `stack` must be a plain file name: empty, containing `/` or `\`, or starting with `.` -> error naming the schema file and the value, before anything is written.
- **Built-in schema.** `specless` loads with zero config: artifacts `blueprint`, `backlog`, `tasks`, apply block tracking `tasks.md`, stack template `stack.md`.
- **new tablet.** Creates `nabu/tablets/<tablet>/.nabu.toml` with `schema = "<name>"`. Creates `nabu/tablets/` when missing. Prints the created path.
- **new --stack.** Same, plus `stack.md` with the exact content of the resolved stack template. Schema without a `stack` key -> refusal, fix: add `stack = "stack.md"` to the schema file.
- **new slice.** Creates `nabu/tablets/<tablet>/<N>-<slice>/`, N = 1 + the highest existing slice number, 1 when none. Writes an empty `.nabu.toml` marker inside. Prints the created path.
- **Refusals** (exit 1, nothing written): invalid name; tablet already exists; `--stack` with a schema that has no `stack` key; slice name already used in that tablet; parent tablet missing; parent tablet not stacked (fix: add `stack.md`); `--stack` on a slice target; a target with more than one `/`.
- **Stacked tablet and slices.** A tablet is stacked when it holds the file its schema's `stack` key names. Its slices are the subfolders named `<N>-<slice>` that hold a `.nabu.toml` marker. Other subfolders are not slices.
- **Lookup.** `<tablet>` resolves to `nabu/tablets/<tablet>/`. `nabu/archive/` is never searched and never listed. A slice resolves by number, by name, or by full folder name. Unknown tablet -> error listing existing tablets. Unknown slice -> error listing that tablet's slices.
- **Errors.** Written to stderr, exit 1, state the cause and a fix the user can paste.

## Out of scope

- Writing artifacts other than `stack.md`. Skills write them.
- Artifact state, progress, `status`, `list` (slice 2). Prompt output (slice 3). `archive` (slice 4). `init` (slice 8).
- Choosing a schema per call (`nabu new --schema`). The config `schema` key covers it.
- Schema graph checks: duplicate ids, unknown `requires`, `build.tracks`, unresolvable artifact templates, cycles (slice 2).
- Rules naming an artifact the tablet's schema lacks (slice 3, checked per tablet when rules are served).

## Gotchas

- The built-in `tasks.md` template names the `openspec-verify` skill. Slice 7 renames it to `nabu-verify`. Until then the old name is correct, because the old skills are still installed.
- A jj workspace inside a nabu repo has no `nabu/` of its own until it gets one => it resolves to the outer nabu root. Workspaces created as siblings (the usual `jj workspace add ../x`) carry their own `nabu/` and resolve to themselves.
- `XDG_CONFIG_HOME` set to an empty string or a relative path counts as unset (XDG Base Directory spec). Tests must set `HOME` too, or the fallback reads the real user config.
- Lookup helpers for existing slices and the schema fields read by later slices (`context`, `rules`, artifacts) carry `#[allow(dead_code)]` with a comment naming the consuming slice. Drop each allow when its caller lands.
- Parse-error snapshots hold the `toml` crate's error snippet. A `toml` upgrade may change them: review, do not blindly accept.
- A broken built-in file is reported as `defaults/...`. Only nabu itself can break it, so the source path is the useful one.
- Fixture slices are named after the reference stack's headings. Fixture text is verbatim reference content and still says "openspec".
- `Config::load` runs in `main::run` before dispatch => a broken config fails every command, even before a name error shows.
- Layers read only `config.toml`, `schemas/<name>/schema.toml`, and `schemas/<name>/templates/*.md`, skipping hidden files. A new layout file (for example a migrate prompt) must be added to `read_layer` explicitly.
