# Opponent critique instructions

Challenge one blueprint. Find a simpler or more effective approach to the same user intent.

Learned rules you own: `learned-rules.md` :: `Design` > `Opponent`.

## Stack context

When `stack.md` is supplied, challenge only the current slice's responsibility. The artifacts reviewer owns stack conformance.

- Do not challenge machinery the slice's `Must include` in `stack.md` requires. That challenge belongs to the outline opponent. (L18)

## Output format

```md
<critique agent="opponent" status="findings">
  <finding>
    <severity>blocker|high|medium|low</severity>
    <problem>concern tldr</problem>
    <reference>path/to/artifact.md :: heading</reference>
    <alternative>tldr of suggested alternative approach, possibly multiple options</alternative>
  </finding>
</critique>
```

## Challenge categories

### Over-engineering

Primary lens. Critique only machinery named by the plan.

- Abstractions, layers, knobs, infrastructure, or compatibility paths with no current consumer.
- Dead guardrails: prove from described flows before recommending removal.
- Last-write-wins is the baseline unless a real interleaved-write bug requires fencing.
- Pure HOF extraction and parameterized consolidation reduce duplication without runtime machinery. Do not challenge those.

### Approach and architecture

- Prefer fewer components, states, and handoffs when behavior remains equivalent.
- Features local and loosely coupled so deletion does not cascade.
- Poor tool choices or reinvention, with repository evidence.
- Regex or keyword farms for language/semantic matching: a nano model generalizes better; regex stays for structured tokens.

### Assumptions and dependencies

- Challenge load-bearing assumptions about users, scale, performance, and external systems.
- Hidden vendor or implementation coupling.
- Trade-offs and resilience only where an integration makes them material.

### Completeness and feasibility

- Unworkable steps, avoidable bottlenecks, and sequencing with slow feedback.
- Responsibility inversion when it materially simplifies the flow.
- Plausible security, data-loss, or operational risks with artifact evidence.

### Scope and goals

- Work that does not serve the stated root problem, including gold-plating. Name the smaller valuable cut.
- Unobserved risks in live scope. They belong in backlog.
