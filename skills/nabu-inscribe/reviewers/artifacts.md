# Tablet artifact critique instructions

Review one tablet or slice artifact bundle for contradictions, missing decisions, and unexecutable work.

Learned rules you own: `learned-rules.md` :: `Design` > `Artifacts`.

## Stack context

When `stack.md` is supplied, check the current slice against it:

- Responsibilities inside `Delivers`, `Must include`, and `Excludes`.
- Dependencies point backward. No cycles.
- References to earlier slices that are not declared outputs.
- Slice folder number that does not match stack position.

## Output format

```md
<critique agent="artifacts" status="findings">
  <finding>
    <severity>blocker|high|medium|low</severity>
    <problem>concern tldr</problem>
    <reference>path/to/artifact.md :: heading</reference>
    <fix>tldr of suggested change</fix>
  </finding>
</critique>
```

## Antipatterns

### Schema conformance

- Structural violations only when they break correctness or traceability.

### Bundle coherence

- Blueprint contract without an objective implementation or verification task. Task without a blueprint contract.
- Duplicate requirements, and scope that diverges from user intent.
- Blueprint decisions too vague to guide work, or locked to needless code-level detail.
- Terminology differences only when they alter scope, traceability, or behavior.

### Executability

- Task without a completion gate.
- Boundary behavior without a test task. Test tasks that pin implementation details instead of boundary behavior.

### Substantive gaps

- Unresolved placeholders, open risks without tasks, and contradictory success or failure contracts.
- Missing ownership or failure propagation at component or system handoffs.
- Performance or scale claims without measurable budgets.
- Persistent-state changes without safe migration and rollback intent.
- Breaking contract changes that do not name affected consumers.
