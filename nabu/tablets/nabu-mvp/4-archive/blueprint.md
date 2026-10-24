# Blueprint nabu-mvp/4-archive

## Feature description

`nabu archive <tablet>` moves a finished tablet out of the active set, into `nabu/archive/YYYY-MM-DD-<tablet>/`. A stacked tablet moves as one folder, `stack.md` and every slice included. Undo is `jj op restore`.

## Agent Decisions

- **Local date.** The date prefix is today's date in the system time zone, from `jiff` (printed as ISO `YYYY-MM-DD`). A UTC date would be off by one day between midnight and about 2:00 in the user's zone. `jiff` (0.2.38, October 2026) reads `TZ` and `/etc/localtime`. When it cannot detect the zone (`TimeZone::try_system` fails), archive prints `warning: system time zone unknown, using the UTC date; set TZ` to stderr and uses UTC => never blocked, never silent.
- **One rename.** The move is a single `fs::rename` of the tablet folder. Same file system => atomic: the tablet is either fully active or fully archived.
- **No schema resolve.** Archive needs only lookup, so a tablet with a broken or unknown schema can still be archived. `config.toml` is still parsed before dispatch (slice 1), so a malformed config blocks archive like every command.
- **Tests fix the date by filter.** Snapshots replace the date with `[DATE]`. The local-date test archives twice, in two temp copies, with `TZ=Etc/GMT-14` and `TZ=Etc/GMT+12`. The zones are 26 hours apart, so their dates always differ: each folder must carry its zone's date, and the two names must differ. Upstream OpenSpec also uses the local date (`formatLocalDate()`). No test-only env var.

## User Decisions

- Whole-tablet archive only. No per-slice archive.
- No guard against unchecked tasks (#2006 out of MVP). `nabu-archive` settles the backlog before it calls `nabu archive`.
- Archived tablets are never searched or listed by other commands.

## Flow

- `archive <tablet>` -> any `/` in the target -> refuse (before lookup) -> lookup tablet -> target = `nabu/archive/<date>-<tablet>` -> create `nabu/archive/` if missing -> exists? refuse -> rename -> print target path. The create comes first so a refusal never changes the tree, and a file named `nabu/archive` fails with a clear cause.

## Contracts

- **Move.** `nabu/tablets/<tablet>/` becomes `nabu/archive/<YYYY-MM-DD>-<tablet>/` with identical content. `nabu/archive/` is created when missing. Prints the new path, root-relative.
- **Refusals** (stderr, exit 1, `error:` then `fix:`, nothing moved).
  - Target containing `/` (slice or deeper) -> whole tablets only, fix: `nabu archive <tablet>`. Checked before lookup.
  - Target exists -> error naming it, fix: rename or remove it.
  - `nabu/archive` cannot be created -> `cannot create nabu/archive: <io error>`, fix naming `nabu/archive`.
  - Unknown tablet -> slice 1 lookup error.
  - Rename fails -> error naming both paths.

## Out of scope

- Backlog handling (`nabu-archive`, slice 7).
- Readiness guard (#2006).
- Archiving across file systems.

## Gotchas

- `fs::rename` replaces an empty target folder on Linux => the explicit existence check before the rename is required.
- `fs::rename` fails across file systems. `nabu/` lives inside one repo, so source and target share one.
