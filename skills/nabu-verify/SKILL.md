---
name: nabu-verify
description: Use when reviewing an implementation against a nabu tablet blueprint, feature description, or explicit comparison ref. Not for checking requirement coverage against an issue or design docs.
---

# Nabu Verify

A non-zero `nabu` exit stops this skill. Show the user its `error:` and `fix:` lines. Never continue on empty output.

Run `nabu` and write artifacts from the repo root: its paths are root-relative.

## Modes

- **Spec:** Default when a tablet or slice exists. Blueprint is intent.
- **Description:** Supplied feature description is intent.
- **Diff:** Branch vs merge target, like a PR preview. User names no target -> default upstream branch: the revset `trunk()`.
- **Rev:** Edits in target jj revision, eg `@`.
- If unclear, ask.

## Setup

- Spec mode: the spec files are the `read:` paths of `nabu instructions build <target>` plus the target's `backlog.md`. Stacked tablet target -> the first slice not `ready to archive` (`nabu status <tablet>`), ask when unclear.
- Read the target repository's `AGENTS.md` and `CODING_STANDARDS.md` (when present) for triage evidence.
- Resolve this skill's directory and every reviewer instruction path to absolute paths.
- Leave reviewer files unread. Only their subagents need those instructions.
- Pass the active workspace and VCS so reviewers inspect the same state.
- Run `codegraph index` in the workspace.

## Lane ownership

Lanes grouped by the files they need => no duplicate traversal. Same root cause from several lanes -> owner lane's wording wins.

- **coverage:** requirement test coverage (owned here => no gap between lanes), test quality, docs, comments, names, literal shape, backlog hygiene.
- **correctness:** behavior, state transitions, performance, reliability, translations.
- **structure:** architecture, dependencies, simplicity, precedent, the full data layer (transactions and caches included). Prompt and type organization only when architecture is the root cause.
- **boundary:** security, all type clarity (types define contracts), contracts, resilience, observability.
- **ui:** user-facing states, flows, accessibility, frontend runtime behavior, parity across frontends.

## Phase 1: Spawn

- jj only: pin the pre-review state: `jj log -r @ -T 'commit_id.short()' --no-graph`. Remember the returned commit id.
- Read the supplied intent. Select one mode before spawning reviewers.
- Spawn all reviewers in parallel exactly once. Skip `ui` when the diff touches no frontend code, and say so in the report.
- Send every lane in one message, each with `run_in_background: false`. You are blocked until all lanes return.
- Spawn every lane as the `nabu-reviewer` agent. Profile unavailable -> `general` for coverage, `explore` for the read-only lanes.
- Reviewer lanes:
  - `coverage.md`: write access for tests, comments, docs, literal shape, and backlog hygiene. Renames come back as findings.
  - `correctness.md`, `structure.md`, `boundary.md`, `ui.md`: read-only = no file writes. Read-only shell queries expected.
- Every reviewer gets `shared-reviewer-rules.md`, `implementer-rules.md`, and only its own instruction file.
- Omit empty fields. Spec files include `stack.md` and `backlog.md` when present. Prefix each file path with `@`.

```text
Role: you are a review subagent. Critique the implementation in your lane.

Instruction files:
- @absolute/path/to/skills/nabu-verify/shared-reviewer-rules.md
- @absolute/path/to/skills/nabu-verify/implementer-rules.md
- @absolute/path/to/skills/nabu-verify/reviewers/<agent_name>.md

Context:
- Scope mode: <spec|desc|diff|rev>
- Input context: <target | desc | diff targets | target rev>
- VCS: <jj|git>
- Worktree: <absolute workspace path>
- [IFF applicable] Compare ref: <merge target, e.g. main. Diff mode, or spec/desc when the implementation spans more than @>
- [IFF applicable] User hints: <actual hints only>

[IFF applicable] Spec files:
- @absolute/path/to/each/spec/file.md
```

- Failed lane => coverage is incomplete. Report the failed lane.
- Harness restart or interruption: completed lanes keep their signal. Relaunch only the aborted lanes, never the completed ones. Do not rerun any reviewers after your fixes.

## Phase 2: Triage critique

1. **Collect**
   - Reviewer autofix edits:

     ```bash
     # this shows all the edits the subagents did
     codegraph sync && jj diff --git --ignore-all-space --from <pre_spawn_commit_id> --to @
     ```

   - Run tests the coverage agent wrote.
   - `status: autofixes` = wrote fixes, no findings. `status: empty` = no findings and no writes. Both are completed lanes.
   - Deduplicate by root cause. Keep the owner lane's wording per [Lane ownership](#lane-ownership).
2. **Verify**
   - Compare code and blueprint against original user intent.
   - Scope narrows search, never silences issue categories.
3. **For every finding**
   - **REJECT:** Existing behavior is correct. Cite the disproving file and symbol. Not for "out of scope."
   - **FIX:** Complete fix, update affected contracts or docs, add focused tests. Fix needs a product guess -> `ASK`, not a partial fix.
   - **ASK:** Product intent is required. Ask one answerable question, record it in backlog, leave code and normative artifacts unchanged. Missing product intent is never `DEFERRED` or `REJECT`.
   - **DEFERRED:** Valid work belongs later without new input. State why and record it in backlog. Not for laziness, style, or "a test file is a lot of work."
4. **Additions and removals**
   - Removes unused complexity: proven -> `FIX`. Guess -> `ASK`.
   - Adds scope (a feature, an option) -> `ASK`.
   - Adds a guard, edge-case, or security handling -> first trace callers, inputs, trust boundaries, deployment, and existing guards. Reachable -> `FIX`. Impossible or already handled -> `REJECT` with the trace. Possible but unobserved -> `DEFERRED`. No speculative code.

## Phase 3: Implement fixes

- Multiple non-trivial fixes -> use `/with-implementer`, mode:=`findings`, review:=`full`, target: each `FIX` item with the owner lane's wording, cited file and symbol, and the required fix. Dispositions, backlog, and blueprint edits stay yours.
- Few small fixes -> fix inline. Spawn overhead would exceed the savings.

## Phase 4: Sync artifacts

Spec mode. Runs the completion gate's artifact-update task (new decisions, gotchas, deferred scope), which must come last.

- Slice-local decisions and gotchas -> this tablet or slice's `blueprint.md`.
- Deferred scope -> `backlog.md`.
- Stack slice: gotchas a later slice needs -> `stack.md` `## Cross-cutting Concerns`. The next slice reads only `stack.md`, never this blueprint. Re-read the later slices' `Must include` first. A note that contradicts one gets caught a slice late.
  - jj: `stack.md` lives in the stack revision, not the slice. Move the edit there, then confirm it landed. Without `--keep-emptied` an emptied slice revision is abandoned.

    ```bash
    # moves the stack.md edit into the stack revision
    jj squash -u --keep-emptied --from @ --to <stack-rev> nabu/tablets/<tablet>/stack.md
    jj diff -r <stack-rev> --summary
    ```
- Unchecked ``Use skill `nabu-verify` `` task exists -> tick it.

## Output format

Order the categories: `REJECT`, `FIX`, `DEFERRED`, then `ASK`. For every item:

```text
- <severity>: short summary of the problem
- <REJECT | FIX | DEFERRED | ASK>: why rejected, what changed, why deferred, what decision is needed
---
```

Then:

- Residual risk and follow-ups in one block.

## Handoff

Stack slice `<tablet>/<N>` with later slices remaining, and no blocking `ASK` items:

- Continue without asking the user to confirm. Output the report first.
- jj: `jj edit` the next slice's pre-created revision. Stop and ask when no unique revision matches.
- End this skill, then continue to `/nabu-inscribe <tablet>/<N+1>`.
- A blocking `ASK` item open -> stop. Offer the next slice after the user answers.
- Only non-blocking / low priority `ASK` items -> backlog them and continue.

Otherwise offer all that apply:

- Top three backlog items, with one recommendation as the most important
- `/nabu-archive` if the tablet is standalone or the stack is fully implemented
- `/nabu-inscribe <tablet>/<N+1>` if later slices remain in the stack
- other options, if any.

## Rules

- Create backlog when `ASK` or `DEFERRED` needs it.
- Use the `renaming` skill. Ignore backwards compatibility: no legacy customers for old names.
- The blueprint can be incomplete or wrong. Expand it when the finding is real.
