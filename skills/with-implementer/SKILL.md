---
name: with-implementer
description: Use when the user or another skill explicitly instructs to invoke this.
---

# with-implementer

- Done-state: the target is implemented.
- Review `full`: you reviewed every edit the implementer made, and valid issues are fixed. Review `deferred`: blockers and dropped intent are resolved.

Why: exploration, test output, and retries stay out of your context. You review a diff you did not write => fresh perspective.

## Brief

- Mode, one kind:
  - `goal`: the user's request, verbatim. Feature, bugfix, refactor, ...
  - `plan`: tablet or slice and its artifacts.
  - `findings`: review findings to fix. Each with its problem, cited file and symbol, and the required fix.
- Review: `full` by default. `deferred` only when the caller passes it.
- Context files: paths that define intent. The caller supplies them, or session context names them.
- Gather only what the implementer cannot find alone: session decisions, constraints, answers to blockers. Leave the implementation path to the implementer.

## Delegate

- Before every spawn or continue, pin a fresh state: `jj log -r @ -T 'commit_id.short()' --no-graph`. Keep the commit id. It snapshots the working copy first => your own edits since the last pin stay out of the implementer's diff.
- Spawn the `implementer` agent. Profile unavailable -> a general writer subagent. Omit `model`.
- Always pass `run_in_background: false`. You are blocked until the implementer returns, because every next step needs its report. Same when you continue it with a later message.
- Later call in the same run: continue the existing implementer session when the harness allows. Its context stays warm. Spawn fresh when its context holds stale intent, such as another change, stack slice, or mode.
  - Continue with `SendMessage` (load it with `ToolSearch` when deferred). It returns at once and the report arrives as a notification, so `run_in_background: false` does not apply. Do your own artifact edits meanwhile, then end the turn and resume on the notification.
- Prompt (fill values, omit empty fields):

  ```text
  Role: Implementer subagent.

  Instruction file: @/absolute/path/to/skills/with-implementer/implementer.md
  Learned rules: @/absolute/path/to/skills/with-implementer/implementer-rules.md

  Context:
  - Worktree: <absolute workspace path>
  - VCS: <jj|git>
  - Mode: <goal|plan|findings>
  - Target: <tablet or slice folder | goal description | findings>
  - [IFF applicable] Parent notes: <session decisions, constraints, answers to blockers, gotchas from earlier runs>

  [IFF applicable] Context files:
  - @/absolute/path/to/each/context/file
  ```

## Review

- Both modes: after subagent returns, run to get all its edits: `jj diff --git --ignore-all-space --from <commit_id> --to @`.
- git mode: `git diff -w` against `git stash create` output taken before spawn (empty -> `HEAD`), plus `git status` for new files.
- `deferred` -> read the diff so you know what changed, then skip the review below. Act on `Blockers`, `Dropped intent`, `Gotchas`.
- `full` -> judge the diff, not the report. Look for:
  - **Plan adherence:** does what the implementation asked. Missing parts, misread intent, unrequested scope, product guesses. `plan`: ticked unimplemented tasks, contract drift from the blueprint.
  - **Correctness:** problems in the new code.
  - **Regressions:** breakage of old contracts.
  - **Loose ends:** stale references, docs, or config the change should have updated. Existing code the change duplicates instead of reusing.
  - **Tests:** changed behavior with a plausible regression and no test that guards it.
- Trust reported checks. Rerun one only when the diff contradicts it.

## Rules

- Small defect -> fix it yourself. Larger -> send concrete corrections to the same implementer and reuse it. Second miss -> fix it yourself.
- You own blockers. Missing product intent -> ask. Unclear failure -> `diagnosing-bug`.
- jj vcs: corrections land in the implementation's revision, not a follow-up.
