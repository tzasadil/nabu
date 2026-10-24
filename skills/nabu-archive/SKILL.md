---
name: nabu-archive
description: Use when archiving finished nabu tablets or completed stacks.
---

# Nabu Archive

Archives whole tablets. A stacked tablet moves with all its slices.

A non-zero `nabu` exit stops the tablet it ran for. Show the user its `error:` and `fix:` lines.

Run `nabu` and write files from the repo root: its paths are root-relative.

## Workflow

1. **Select**
   - Prefer explicit names or the session tablet.
   - No target -> `nabu list`. Prefer the only active tablet, then the only one marked `ready to archive`. Otherwise ask with multi-select.
   - Slice target (`<tablet>/<slice>`) -> name the tablet that would move and ask whether to archive it whole.
   - Announce the resolved set.
2. **Check state.** `nabu status <tablet>` for each. Not `ready to archive` -> show what is open and ask before going on.
3. **Settle the backlog.** Read the tablet's `backlog.md`, or every slice's `backlog.md` for a stack. Propose one disposition per item:
   - **keep:** stays in the archive.
   - **move:** into a new tablet. Name it now, create it in step 5.
   - **drop:** discarded.
4. **Approve.** Ask once for the whole set: tablets to archive plus every item disposition. Archiving removes tablets from the active workflow.
5. **Archive each tablet.** `nabu archive <tablet>` first. Only for a tablet that archived, run its `move` writes (`nabu new`, then the item text). A failed tablet is reported with its `move` writes skipped, and the rest continue. Never move files yourself.
6. **Report.** Archived tablets with their new paths, moved backlog items with their new tablets for archived tablets only, failed tablets with causes and the skipped moves.
