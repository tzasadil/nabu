---
name: nabu-tweak
description: Use when the user changes requirements for a tablet whose implementation is already in progress.
---

# Nabu Tweak

## Sources of truth

- Artifact presence: `nabu status <target>`
- Paths, dependencies, schema rules: `nabu instructions <artifact-id> <target>`
- A non-zero `nabu` exit stops this skill. Show the user its `error:` and `fix:` lines. Never continue on empty output.
- Run `nabu` and write artifacts from the repo root: its paths are root-relative. Prefix the root to pass reviewers absolute paths.
- Behavior conflicts: user request, `blueprint.md` contracts, then `tasks.md`
- `backlog.md` is non-normative deferred work

## Workflow

1. **Select**
   - Prefer a supplied target (`<tablet>` or `<tablet>/<slice>`) or the session tablet.
   - No target -> `nabu list`, then ask.
   - Stack request: read `stack.md`, map the requested behavior to its owning slice.
   - Mismatched supplied slice -> retarget to that owner. Ask when ownership is ambiguous or spans slices.
   - jj: record the current revision. `jj edit` the owner's unique revision before touching artifacts or code. No unique match -> stop. Child revisions inherit the edits and may conflict.
2. **Map the request**
   - Named artifacts before nearby code. Implementation does not redefine intent.
   - Behavior in `blueprint.md` `## Contracts`. A contract that lives only in `## Agent Decisions`, `## User Decisions`, or `## Flow` is hidden.
   - Boundaries and design in `## Agent Decisions`, `## Flow`, or `## Gotchas`.
   - A tweak that changes a design choice is a user decision. Record it in `## User Decisions`. Rewrite the entry it supersedes in either Decisions section. Never keep both.
   - Execution and focused checks in `tasks.md`.
3. **Update artifacts**
   - `blueprint.md` before `tasks.md` so execution follows the contract.
   - Add a corrective task instead of rewriting a completed task. Preserves history.
   - Update `stack.md` when slice scope or responsibility changes. jj: it lives in the stack revision. Edit it there, or move a slice-side edit with `jj squash -u --keep-emptied --from @ --to <stack-rev> <path>`.
4. **Restore coherence.** Re-read touched artifacts. Fix contradictions before product code.
5. **Implement** the smallest code and test changes that satisfy the updated artifacts.
   - Small change (a few edits in one area) -> implement inline. Spawn overhead would exceed the savings.
   - Larger -> delegate the new and corrective tasks (see Delegation), then continue here.
   - Loop back to artifacts when coding exposes another intent mismatch.
6. **Finish**
   - Focused tests, lint, type checks.
   - Mark tasks complete only after their checks pass.
   - After a stack-slice tweak, return to the recorded jj revision.
   - Report checks, omissions, and blockers. Do not narrate artifact edits.

## Delegation

Run `/with-implementer` with:

- Mode `plan`, target: the tablet or slice folder.
- Context files: every path in the `read:` line of `nabu instructions build <target>`.
- Parent notes: the requested tweak, the new and corrective task ids to implement, answers to blockers.

Then:

- Design mismatch -> update artifacts, then resume with the change in `Parent notes`.
- Fold the implementer's `Gotchas` into blueprint `## Gotchas` when later work needs them.

## Boundaries

- Ask when intent conflicts or the request crosses the current blueprint or slice responsibility. A request that is a new effort: ask whether to split.
- Direct tweak loop. Do not auto-run inscribe's refine pass or `/nabu-verify`.

## Stack example

User on slice 3 of stack `foo`: "draft projects need auto-save every 10 seconds." Tweak is confined to slice 2.

- select `foo/3`
- confirm stack `nabu/tablets/foo/stack.md`
- map to slice 2
- `jj edit <slice_2_rev>`
- update `blueprint.md` + `tasks.md`, then the stack (`jj squash -u --keep-emptied` to the stack revision)
- implement (edits propagate to child revisions)
- focused tests/lint
- `jj edit <slice_3_rev>` back
- report
