---
name: nabu-outline
description: Use when a net-new feature is too large for one reviewable PR and needs design plus vertical slicing.
---

# Nabu Outline

## Gates

- Planning artifacts only. Application code, `/nabu-build`, and implementation skills stay outside this workflow.
- Setup: run `codegraph index`.
- A non-zero `nabu` exit stops this skill. Show the user its `error:` and `fix:` lines. Never continue on empty output.
- Run `nabu` and write artifacts from the repo root: its paths are root-relative. Prefix the root to pass reviewers absolute paths.

## Workflow

1. **Audit reuse.** Inspect code, `AGENTS.md`, docs, and recent history yourself. No subagent. Record reusable foundations and real gaps with paths.
2. **Grill the user** relentlessly, one question at a time, until every material ambiguity is resolved. For each big decision, offer two or three viable approaches and mark one as recommended. `Defaults` in `learned-rules.md` settle recurring questions. Use `AskUserQuestion` tool.
3. **Present the design.** Boundaries, components, data flow, failure behavior, testing intent. Each slice: name, responsibility, dependency order. Align with the user section by section.
4. **Design the stack.** Sequential vertical slices, each easy to review without asking whether to split. Ground each slice in the reuse audit. Design within [learned-rules.md](../nabu-inscribe/learned-rules.md). Then write it:
   - IFF `.jj/` exists: pick the stack revision before the first write. `@` empty -> `jj describe -m "<tablet>: stack"`, else `jj new -m "<tablet>: stack"`.
   - `nabu new <tablet> --stack`, then write `stack.md` from `nabu instructions stack <tablet>`.
5. **Run opponent critique.**
   - Resolve this skill's directory to `{skillRoot}`.
   - Spawn one `nabu-opponent` agent. Profile unavailable -> a read-only explore agent. Give it only the design or absolute `@path/to/stack.md` path, the source (issue, design docs) when one exists outside chat, plus `@{skillRoot}/reviewers/opponent.md`.
   - `empty` = clean.
   - Resolve every finding with a fix, evidence-backed pushback, or a user decision.
   - Re-present changed design sections. Return to exploration when critique changes the approved direction.
6. **Get approval.** Concise design summary. Ask for explicit approval.
7. **Scaffold the slices.** `nabu new <tablet>/<slice>` per slice in N order, matching the `### <N>.` headings. IFF `.jj/` exists, one revision per slice after that. The stack revision keeps `stack.md` and every slice marker, slice revisions start empty.

   ```bash
   # use to create one revision per planned slice; loop for each slice
   jj new -m "<tablet> <N>: <slice-name>"

   jj log --no-graph -r "ancestors(@, 10)"
   jj edit <first-rev-id>
   ```

## Stack planning

- Default to multiple thin vertical slices. One change only when splitting adds more coordination than value.
- Each slice: a complete path through every relevant layer with an observable outcome.
- Outcomes, contracts, and decisions only. Implementation detail belongs to the slice's blueprint or to build.
- Cleanup stays inside the slice that creates the need. A separate cleanup slice needs standalone value.
- Each slice has a `Depends on` line naming the earlier outputs or contracts it consumes, or `nothing in this stack`. Dependencies point backward.
- Size for one focused work session: two or more parts a reviewer could verify alone (a table with its rules, a route group, a screen) -> split, unless they only make sense together.
- Easy to review on its own: one understandable diff with its tests and observable outcome. Full independent deployment or standalone demoability is not the goal. A slice may assume its predecessors are merged.
- Kebab-case tablet name. It maps to the branch names the stack ships under.

## Handoff

After approval and the jj chain, end this skill, then continue to `/nabu-inscribe <tablet>/1`. No confirmation.
