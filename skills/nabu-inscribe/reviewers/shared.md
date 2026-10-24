# Shared reviewer rules (inscribe)

Read this file first, then your own instruction file.

## Source of truth

- Conflicts: user request, blueprint contracts, then tasks.
- Backlog is non-normative. Deferred work cannot rewrite scope.

## Workflow

1. Read the supplied user request, the full bundle, `AGENTS.md`, and `../learned-rules.md` (relative to this file).
2. **Primary:** review every criterion in your instruction file.
3. **Secondary:** check the learned rules under the heading your instruction file owns. A broken rule without a recorded deviation is a finding. Cite its `(L<n>)` ID in `<problem>`. The rules extend your criteria, never replace them.
4. Anchor findings to a file and stable heading: `path/to/artifact.md :: heading`.

## Stack context

When `stack.md` is supplied, review only the current slice. `stack.md` is dependency and responsibility context, not a review target.

## Uncertain claims

- Check load-bearing platform, API, and vendor claims against local evidence first.
- Claim still unconfirmed -> finding. Start `<problem>` with `PARTIALLY_SUPPORTED` plus its constraints, `UNSUPPORTED`, or `UNCERTAIN` plus the missing evidence.

## Hard rules

- Return only parseable XML inside one `md` fence.
- No findings: `<critique agent="<agent>" status="empty"/>`.
- Invalid input: `<critique agent="<agent>" status="error" reason="<input required to proceed>"/>`.
- Remain read-only and non-interactive.
