# Shared Reviewer Rules

Read this file first, then your own lane file. `<skill-dir>` = the directory holding this file.

`implementer-rules.md` holds the rules every implementer got. Rules your lane owns are review criteria too: a broken one is a finding.

Do not report changes that land while one request runs (access revoked, record edited, row deleted). The window is seconds, and the next request sees the new state. Queued and long-running work (jobs, syncs) stays in scope.

## Review modes and scope

The parent supplies one scope mode:

- `spec`: implementation of a tablet or slice. The supplied spec files are intent.
- `desc`: implementation of a supplied feature description. The description is intent.
- `diff`: the branch's changes against the supplied Compare ref (merge target), like a PR. No intent artifact.
- `rev`: edits in the supplied target jj revision. No intent artifact.

Rules:

- Work only in the supplied workspace.
- Start from target-implied files. Scope narrows search, never silences issue categories. Widen only from a concrete file and symbol. Never a whole-repo audit.
- Spec mode: skip findings the supplied `backlog.md` already tracks. Other modes have no backlog.

## Concurrent writers

The `coverage` agent runs beside you. It writes tests, fixtures, comments, docs, string-literal shape, and backlog lines. It never changes behavior or names => its edits cannot invalidate your findings. Expect files to change under you.

## Changed line scan

Derive changed paths from the VCS. Never a hand-maintained file list.

Run diff scans through the skill script. Never hand-roll a `jj diff` pipeline: `jj` parses path args as filesets => raw paths break on `()` in Next.js route groups.

```bash
# git-format diff of non-test, non-lockfile paths
<skill-dir>/scripts/diff-scan full
# compact index of added lines
<skill-dir>/scripts/diff-scan lines
```

- Compare ref supplied: prefix every scan with `REF=<compare-ref>` (the merge target). Scan shows the branch's own changes since the fork point, working-copy edits included, like a PR preview.
- No Compare ref: the scan covers the working-copy revision `@`.
- `full` drops lockfiles. `nabu/**` paths stay. `lines` also drops `nabu/**` paths.
- Both modes omit tests by default. Add `--include-tests` when a finding's evidence is in a test file. Test deletions are review surface too => run `full --include-tests` before judging test coverage.
- Treat either stream as an index. Inspect deletions and surrounding code before filing findings.

## Abide by `AGENTS.md`, `CODING_STANDARDS.md`, and user rules

- `CODING_STANDARDS.md` at the repo root, when present: judgement-call rules written for review.
- Apply only conventions relevant to your lane.
- Cite policy findings with a stable section reference (`[AGENTS.md#Testing]`).

## Investigation

- Scale depth to what the change can destroy. Destructive or irreversible paths, lifecycle ownership, persisted contracts and schemas, auth boundaries, integration boundaries, and concurrency over shared state each get an explicit attempt to break the invariant they rest on. A prose-only change gets the minimum.
- Do not run tests, lint, or formatters. Run a type check (`tsc --noEmit` or the project equivalent) only when one specific finding hinges on a type fact that reading the code cannot settle. Never as a routine or batched step.

## Hard rules

- Report every evidence-backed root cause you spot, in your lane or not (the correct lane may not spot the bug). Do not hunt for problems outside your lane.
- Section bullets either describe the expected state or name a defect. Either way, flag changed code that breaks the expected state.
- Never ask questions (session not interactive). Missing context blocks the whole review -> `status: error`. One unverifiable claim -> drop it.
- Remain read-only unless your instruction file grants write permission.

## Verification loop (before finalizing)

For each finding, confirm:

- The code proves the claim at a real path and symbol.
- No stronger finding already covers the same root cause.
- Spec mode: the backlog does not already track it.
- It is not an excluded nit: unused imports, test magic numbers, explained lint disables, contract-preserving boundary translation.
- `hint`, when present, is a concrete fix. Omit vague advice.

Drop any finding that fails.

## Output format (mandatory)

Your final message is the critique and nothing else. Plain Markdown. Copy the structure below without its code fence. The first line is the header.

```md
# critique: <agent_name>

status: findings

## <severity>: <one-line problem>

- loc: `<path>::<symbol>`
- spec: <plan step, requirement id, or AGENTS.md section>
- evidence: <short proof: caller, type, upstream guarantee, failing scenario>
- hint: <concrete fix>
```

- One `##` block per finding. `loc` is required. Omit `spec`, `evidence`, and `hint` when they do not apply.
- The `##` line is one sentence. Put proof in `evidence`, not in the problem line.
- Severity is your judgment (`high`, `medium`, `low`). Only the gap between high-impact defects and polish must be clear.
- `loc` is `<path>::<symbol>` or `<path>`. Omit line numbers so locations survive concurrent edits.
- `status` is one of:
  - `findings`: one or more findings follow.
  - `autofixes`: you wrote fixes and have no findings. Header only.
  - `empty`: no findings and no writes. Header only.
  - `error: <reason>`: not enough context to review. Header only.
