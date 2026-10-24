---
name: nabu-build
description: Use when implementing or continuing tasks from a nabu tablet or one stack slice.
---

# Nabu Build

You orchestrate. An implementer subagent writes the code.

A non-zero `nabu` exit stops this skill. Show the user its `error:` and `fix:` lines. Never continue on empty output.

Run `nabu` and write artifacts from the repo root: its paths are root-relative.

## Workflow

1. **Select**
   - Target (`<tablet>` or `<tablet>/<slice>`) already established in this session -> use it.
   - No session target -> `nabu list`, then ask.
   - Stacked tablet (`nabu list` shows `N slices`) -> `nabu status <tablet>`, take the first slice not `ready to archive`. Ask when unclear. `nabu instructions build` needs the slice.
2. **Load**

   ```bash
   # use to load artifact context and the build instruction
   nabu status <target>
   nabu instructions build <target>
   ```

   - Ensure you have read every path in `read:`.
   - Blocked -> report the missing artifact. All tasks done -> skip to handoff.
   - Announce the target and `N/M` progress before implementation.
3. **Delegate**
   - Run `/with-implementer` with mode:=`plan` and review:=`deferred`. Handoff to `/nabu-verify` handles the review.
   - Fold the implementer's `Gotchas` into blueprint `## Gotchas` when later work needs them.

## Handoff

- After all tasks finish, end this skill, then continue to `/nabu-verify`. No confirmation. Implementation is incomplete until reviewed.
- The next skill owns the final response and next steps. No implementation summary, no archive suggestion.
