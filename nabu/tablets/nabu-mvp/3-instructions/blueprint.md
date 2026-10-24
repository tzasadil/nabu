# Blueprint nabu-mvp/3-instructions

## Feature description

`nabu instructions` prints the prompt text a skill feeds to the model. It is the single source of templates, schema instructions, context, and rules => no skill keeps its own copy.

- `nabu instructions <artifact> <target>`: how to write one artifact of a plain tablet or a slice.
- `nabu instructions build <target>`: how to implement a plain tablet or a slice: what to read, what is pending.
- `nabu instructions stack <tablet>`: how to write the tablet's stack file.

The slice also adds the two checks the stack assigned here: rules naming unknown artifacts, and `generates` as a plain file name.

## Agent Decisions

- **Rules keys.** A `[rules]` key is valid only when it is an artifact id of the tablet's schema. No reference config sets rules for apply or stack, so those prompts carry no rules block (see backlog).
- **Rules keep their source file.** `Config` keeps `rules` as it is and gains a sibling map: key -> config files that set it, filled in the same merge loop. The unknown-key error names every file that sets the key. Checked per tablet by all three prompt kinds, not at config load: different tablets may use different schemas.
- **Blocked artifacts still print.** The header says `state: blocked` and `requires` shows what is missing. A skill may need the template early. Refusing would only hide it.
- **Read list.** One rule: the done artifacts in the transitive `requires` closure of a set of ids, in schema order, plus the stack file when the target is a slice. The artifact prompt passes its own `requires`. Apply passes `build.requires`. The non-normative backlog stays out of apply's list unless the schema makes apply require it => deferred work never widens scope.
- **Pending lines share the progress parser.** The `report.rs` task parser returns unchecked lines in the same pass as the counts. Pending is empty when the tracks file is not done, matching progress `0/0`.
- **Reuse slice 2 owners.** Target parsing (`tablets::split_target`), stacked check (`tablets::stack_file`), artifact states and progress (`report::load`, `report::read`), file reads (`layers::read_file`). Drop the `#[allow(dead_code)]` on `Config.context`, `Config.rules`, `Artifact.description`, `Artifact.instruction`, `Apply.instruction`: this slice reads them. `Schema.name` and `Schema.description` stay accepted, never-printed keys: keep their allows, commented as documentation only. The rules-key error names the schema by the tablet's `.nabu.toml` name plus the schema file path, matching what `status` prints.

## User Decisions

- Header of `key: value` facts, then prose in XML-style tagged blocks. Tags give unambiguous boundaries when a template holds its own headings or code fences. Empty blocks are omitted.
- Output is the prompt text itself. No `--json`.
- Every template and rule comes from the layers. Context and rules: built-in, then global, then repo.

## Flow

- `instructions <artifact> <target>` -> split target -> lookup -> resolve schema (graph, `generates`) -> check rules keys -> refuse stacked root -> read artifact states -> print header + blocks.
- `instructions build <target>` -> same lookup and rules-key check -> states + progress + pending lines -> print.
- `instructions stack <tablet>` -> lookup tablet and rules-key check -> schema must have `stack` -> print.

## Contracts

- **Block format.** `<name>` on its own line, the text with surrounding blank lines trimmed, `</name>` on its own line, one blank line between blocks. A block with empty text is omitted. Text is verbatim otherwise, except line endings are normalized to LF.
- **Context block.** Each layer's `context` trimmed, joined by one blank line, built-in first, then global, then repo.
- **List values in the header.** Comma-separated, `none` when empty. Paths are root-relative.
- **Artifact prompt.** Header lines in this order: `artifact: <id>`, `target: <tablet>[/<N>-<slice>]`, `write: <path of generates>`, `state: done|ready|blocked`, `requires: <id> (done|missing), ...`, `read: <paths>`. Blocks in this order: `description`, `instruction`, `context`, `rules` (one `- <rule>` line each), `template`.

  ```text
  artifact: tasks
  target: nabu-mvp/3-instructions
  write: nabu/tablets/nabu-mvp/3-instructions/tasks.md
  state: ready
  requires: blueprint (done)
  read: nabu/tablets/nabu-mvp/3-instructions/blueprint.md, nabu/tablets/nabu-mvp/stack.md

  <description>
  Compact checklist with completion gates and artifact sync
  </description>

  <instruction>
  Section 'completion gate' is mandatory ...
  </instruction>

  <template>
  # TODO Checklist
  ...
  </template>
  ```

- **Apply prompt.** Header: `build: <target>`, `state: blocked|ready|all done`, `progress: <c>/<t> tasks`, `requires: <id> (done|missing), ...` from `build.requires`, `tracks: <path>`, `read: <paths>`. Blocks: `instruction` (`build.instruction`), `context`, `pending` (the unchecked task lines of the tracks file, verbatim with LF endings, in file order). `blocked`: some `build.requires` artifact not done. `all done`: total > 0 and every task checked. Else `ready`.
- **Stack prompt.** Header: `stack: <tablet>`, `write: <path of the stack file>`, `state: done|ready`. Blocks: `context`, `template` (the stack template). Works on a plain tablet too: that is how it gains a stack file.
- **Refusals** (stderr, exit 1, `error:` then `fix:`).
  - Unknown artifact id -> error listing the schema's artifact ids.
  - Artifact or apply target is a stacked tablet's root -> artifacts live in slices, fix: `nabu instructions <same> <tablet>/<N>`, error lists the slices.
  - `stack` with a slice target, or with a schema that has no `stack` key -> refusal naming the cause.
  - A `[rules]` key that is not an artifact id of the tablet's schema -> error naming every config file that sets it, the key, and the schema (`.nabu.toml` name and schema file). All three prompt kinds check it.
  - A `generates` value that is empty, contains `/` or `\`, or starts with `.` -> error naming the schema file and the value. Checked in `schema::resolve`, so `new`, `status`, and `list` refuse such a schema too. No fixture or built-in schema is affected: all use plain names.
  - Slice 1 and 2 lookup errors apply unchanged.

## Out of scope

- Phase guidance beyond apply (upstream #1913: `instructions archive`). `nabu-archive` reads the backlog itself.
- Rendering placeholders inside templates. Templates are printed as stored.
- `--json`.

## Gotchas

- The built-in `tasks.md` template and the `apply` instruction still name `openspec-verify`. Slice 7 renames them (recorded in its stack Delivers).
- An artifact named `apply` or `stack` is shadowed by the subcommand word. No schema does this. See backlog.
- Context and rules are merged per repo, not per tablet. Only the rules-key check depends on the tablet's schema.
