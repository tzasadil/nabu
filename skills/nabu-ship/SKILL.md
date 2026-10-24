---
name: nabu-ship
description: Use when opening or updating stacked GitHub PRs for a nabu stack, a plain tablet, or any range of jj revisions.
---

# Nabu Ship

Opens one GitHub PR per revision, each based on the revision below it. A rerun after edits pushes the restacked revisions and reuses the open PRs.

- `{skillRoot}` is this file's directory.
- A non-zero script exit stops this skill. Show the user its `error:` and `fix:` lines.

## Workflow

1. **Select the range.**
   - Stack: the slice revisions `nabu-outline` created, described `<tablet> <N>: <slice>`. Bottom = slice 1, top = the highest slice to ship. The stack revision below slice 1 rides along in PR 1 => reviewers see `stack.md` there.
   - Plain tablet or one revision: the same revision as bottom and top.
   - Base: the bookmark the user names, else the trunk bookmark (`trunk()`). Ask when unclear.
2. **Preview.** Show the user the printed PR chain.

   ```bash
   # use to preview the bookmarks and the PR chain; pushes and creates nothing
   python3 {skillRoot}/scripts/stack_pr.py -t <base bookmark> -s <bottom rev> -e <top rev> --dry-run
   ```

3. **Approve.** Ask once. Pushes and PRs are public, and `jj op restore` cannot take them back.
4. **Ship.** Same command without `--dry-run`. Report the PR URLs.

## After edits

jj rebases every slice above an edited one => rerun the same command. It pushes every restacked slice again. Open PRs keep their title, body, and base.
