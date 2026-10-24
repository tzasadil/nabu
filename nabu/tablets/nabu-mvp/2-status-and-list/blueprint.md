# Blueprint nabu-mvp/2-status-and-list

## Feature description

Two read-only commands that report where planned work stands:

- `nabu status <tablet>` or `nabu status <tablet>/<slice>`: each artifact as done, ready, or blocked, plus task progress. A stacked tablet shows the stack file, then one block per slice.
- `nabu list`: one line per active tablet with task progress, slice count for stacked tablets, and a "ready to archive" mark.

The slice also adds the schema graph checks that slice 1 deferred, because `status` is the first command that reads the artifact graph.

## Agent Decisions

- **Template comparison.** Both texts are normalized first: line endings to `\n`, surrounding whitespace trimmed. An agent that only touches whitespace has not done the artifact.
- **Checkbox syntax.** A task is a line matching optional indent, a list marker (`-`, `*`, `+`), a space, then `[ ]`, `[x]`, or `[X]`. Nested checkboxes count. This matches the reference `tasks.md` files.
- **Graph checks inside `schema::resolve`** (`src/schema.rs`), next to the slice 1 `stack` check. Every command, current and future, gets them => no caller can forget. A broken schema stops `new` too, like a broken config already does.
- **Reuse slice 1 owners.** Move the target split from `src/new.rs::parse_target` into `src/tablets.rs` (tablet plus optional slice key, the "more than one `/`" error, name checks left to callers). One `stack_file` helper there decides whether a tablet is stacked, used by `new` and `status`. Artifact and tracks reads use `layers::read_file` (made crate-visible): not found -> none, any other failure -> error naming the file.
- **Dead-code allows per field.** Drop the allows on what `status` and `list` read (`id`, `generates`, `template`, `requires`, `build.requires`, `build.tracks`, `find_slice`). Keep field-level allows naming slice 3 on `Schema.name`, `Schema.description`, `Artifact.description`, `Artifact.instruction`, `Apply.instruction`, `Config.context`, `Config.rules`.
- **Command outcome.** A command returns its stdout text plus an exit code, so `list` can print every row and still exit 1. Errors that stop a command keep the slice 1 path (stderr, exit 1).

## User Decisions

- Aligned columns, one compact text format for humans and agents.
- Done = file exists, is not empty, and differs from its template (#1084).
- Progress counts checkboxes only in the schema's `build.tracks` file. `backlog.md` never counts.
- Ready to archive = every tracked task checked, in every slice (#1831).
- Slices are the `<N>-<slice>/` folders holding the `.nabu.toml` marker. `nabu-outline` scaffolds every planned slice, so the folders are the full plan.
- `status` also accepts `<tablet>/<slice>` and prints only that slice under the tablet header. Skills work one slice at a time.

## Flow

- `status <target>` -> lookup tablet (and slice) -> read `.nabu.toml` -> resolve schema -> graph checks -> per folder: artifact states in schema order + progress -> print.
- `list` -> list active tablets -> per tablet: resolve schema, graph checks, progress, slice count -> one row each -> exit 1 if any row is an error.

## Contracts

- **Artifact state.** `done`: the `generates` file exists, is not empty after normalizing, and differs from its normalized template. `ready`: not done, and every `requires` id is done. `blocked`: not done, and some `requires` id is not done. A blocked line ends with `needs <id>, <id>`: only the unfinished ids, in the artifact's `requires` order.
- **Artifact order.** Schema declaration order.
- **Progress.** `<checked>/<total> tasks`, counted in the folder's `build.tracks` file only when the artifact generating it is done. Missing or not done -> `0/0 tasks` => template placeholder tasks never count. A stacked tablet's header sums all its slices.
- **Reads.** Only "not found" counts as missing. Any other read or listing failure (permission, invalid UTF-8, a folder where a file should be) -> error naming the file. In `status` it stops the command. In `list` it becomes that tablet's error row. A stack file or slice marker that exists but is not a regular file is an error too, never a plain tablet or a missing slice. A folder name that is not UTF-8 in a tablet or slice listing is an error naming the parent folder.
- **Ready to archive.** Plain tablet: total > 0 and checked = total. Stacked tablet: at least one slice, and every slice has total > 0 and checked = total. The mark is the text `ready to archive`.
- **Table rule.** Each table pads every column to its widest cell, two spaces between columns. Absent optional cells are blank. Trailing whitespace is trimmed on every line. The tables: the artifact lines of one folder, and the rows of `list`. Header lines and slice header lines are not padded. An error row of `list` is not a cell row: its first cell joins column 1, and `error  <cause>` follows unpadded. It never widens a column.
- **Plain tablet output.** Header `<tablet>  <schema>  <c>/<t> tasks[  ready to archive]`, then the artifact table: `<id>  <state>[  needs ...]`. `<schema>` is the name in the tablet's `.nabu.toml`.

  ```text
  42-empty-list-notice  specless  7/7 tasks  ready to archive
  blueprint  done
  backlog    done
  tasks      done
  ```

- **Stacked tablet output.** Header `<tablet>  stacked  <schema>  <c>/<t> tasks[  ready to archive]`. Next line `<stack file>  done|ready`, judged by the same content rule against the stack template. Then per slice in N order, after one blank line: `<N>-<slice>  <c>/<t> tasks[  ready to archive]`, then its artifact table indented two spaces.

  ```text
  nabu-mvp  stacked  specless  18/18 tasks
  stack.md  done

  1-config-and-new  18/18 tasks  ready to archive
    blueprint  done
    backlog    done
    tasks      done

  2-status-and-list  0/0 tasks
    blueprint  ready
    backlog    blocked  needs blueprint
    tasks      blocked  needs blueprint
  ```

- **Slice target output.** The stacked header, the stack file line, then only the named slice's block.
- **List output.** Active tablets sorted by name, one table: `<tablet>  <c>/<t> tasks  [<k> slices]  [ready to archive]`. No tablets -> `no active tablets`. `nabu/archive/` is never listed.

  ```text
  42-empty-list-notice  7/7 tasks              ready to archive
  nabu-mvp                18/18 tasks  2 slices
  ```

- **List failures.** A tablet whose `.nabu.toml`, schema, graph, or files fail prints the row `<tablet>  error  <first line of the cause>`, without the `error:` prefix. The tablet name is padded to the first column's width, the rest is unpadded (see `Table rule`). The other rows still print. Everything goes to stdout, nothing to stderr. Exit 1 when any row is an error, else 0. The full message: `nabu status <tablet>`.
- **Graph checks.** Duplicate artifact id, unknown id in `requires` or `build.requires`, `build.tracks` not generated by any artifact, artifact template resolvable in no layer, dependency cycle -> error naming the schema file and the offending ids or file. A cycle names the ids on it.
- **Refusals** (stderr, exit 1). Slice 1 lookup errors for unknown tablets and slices. A slice target on a tablet that is not stacked -> cause names the missing stack file, fix: add it. A target with more than one `/` -> invalid target. A schema without a `stack` key makes every tablet of that schema plain.

## Out of scope

- Printing paths or prompt text. Slice 3 (`instructions`) prints output paths.
- `--json`. One text format.
- Showing backlog item counts. `nabu-archive` reads `backlog.md` itself (slice 7).
- `depends_on` between tablets (#1915), drafts (#1818).

## Gotchas

- A stacked tablet ignores artifact files at its own root. Only slices carry artifacts.
- A plain tablet ignores subfolders, even ones that look like slices.
- `backlog` in the built-in schema requires `blueprint`, so it shows `blocked  needs blueprint` until the blueprint is done, then `ready` until written. Expected for an optional artifact.
- A read error in any slice stops `status <tablet>/<slice>`: the tablet header sums every slice, so every slice is read.
