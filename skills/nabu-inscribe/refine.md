# Refine

Critique one proposed tablet or slice, resolve verified findings, leave its artifact bundle coherent.

## Setup

- `{skillRoot}` is this file's directory.
- Resolve absolute reviewer paths under `{skillRoot}/reviewers/`.
- Run `nabu` and write artifacts from the repo root: its paths are root-relative. Prefix the root to pass reviewers absolute paths.
- Target only the current folder (a plain tablet or one slice): the directory of the `tracks:` file in `nabu instructions build <target>`. For a slice, also read and pass the absolute `stack.md` path as dependency context.
- Resolve a user-supplied artifact file or pointer to its parent folder.
- Pass the active jj workspace so reviewers inspect the same files.

## Sources of truth

- Resolve artifact paths from the `write:` and `read:` lines of `nabu instructions` (`nabu status` prints none).
- Conflict precedence: user requirement, `blueprint.md` contracts, then `tasks.md`.
- `backlog.md` is non-normative. Deferred work cannot silently rewrite contracts.

## Reviewer contract

- Spawn each reviewer read-only, once per phase run.
  - Artifacts: `nabu-artifacts` agent. Fallback: explore agent.
  - Opponent: `nabu-opponent` agent. Fallback: explore agent.
- Spawn strictly in order: artifacts, resolve, opponent, resolve. Never in parallel: the opponent must critique the fixed bundle.
- Pass only the target paths, the user request text, and the absolute paths of `{skillRoot}/reviewers/shared.md` and the reviewer's instruction file.
- Fenced `md` containing parseable XML with `<critique agent="..." status="findings|empty|error">`.
- `empty` = clean.

## Phase 0: Bundle check

Reviewers spent findings on task paths, commands, and `UNCERTAIN` placement a script can see (L15, L16). Run it first, fix every error, rerun until clean:

```bash
# use to catch unresolved task paths, commands without a working directory, dead jj IDs, misplaced UNCERTAIN
python3 {skillRoot}/scripts/check_bundle.py <folder>
```

- Exit 2 = wrong folder or no `tasks.md`. Build `tasks.md` or pass the slice folder, then rerun.
- A path a task creates passes when its line says "new", "add", or "create" and the parent directory exists. Any other unresolved path is a typo or a stale premise.

## Phase 1: Artifact critique

- Reviewer contract with `{skillRoot}/reviewers/artifacts.md`.
- Stack mode: judge current-slice responsibility, dependencies, cross-references, and slice numbering against `stack.md`.

## Phase 2: Resolve (edit artifacts)

1. Run both commands, then read every path in the `read:` line:

   ```bash
   # use to load artifact paths and build-instruction context
   nabu status <target>
   nabu instructions build <target>
   ```

2. Parse each `<finding>`. One internal disposition:
   - **REJECT:** evidence proves the finding wrong. Leave artifacts unchanged.
   - **FIX:** smallest coherent artifact edit. Unobserved speculative concern -> backlog only.
   - **ASK:** product intent is missing. Default here when even slightly unsure. Ask one answerable question. Leave that finding unchanged.
3. Behavioral fixes: update blueprint contracts and decisions before `tasks.md`. Blueprint is not a requirements dump. Tasks are not a design doc.
4. Re-read the bundle for contract, flow, task, and terminology consistency.
5. The Phase 0 bundle check. Fix and rerun until clean, or report a tool/schema blocker.

## Phase 3: Opponent critique

- Run the reviewer contract with `{skillRoot}/reviewers/opponent.md`.
- Opponent findings use `<alternative>` instead of `<fix>`.
- Immediately repeat Phase 2 for opponent findings.

## User-facing output

- One short status line per review, or combine both.
- `ASK:` block only for unresolved user decisions.
- Keep internal dispositions out of the user-facing response.

## Handoff

- Both reviews resolved, no unresolved `ASK:` -> end this skill, then continue to `/nabu-build <target>`. Default in every cycle and every slice. Stack mode: build the current slice only, and name it.
- Unresolved `ASK:` -> stop and ask. After the answer, apply it, rerun the bundle check, then continue to `/nabu-build` without offering.
- Several tablets in one request -> inscribe, build, and verify each in order.
