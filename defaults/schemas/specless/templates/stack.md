# Stack: <stack-name>

## Overview
<what the whole stack delivers as a whole>

## Source requirements
- <one requirement from the source (user request, issue acceptance criteria, design docs if any)> -> <slice N, or the Decisions line that drops or defers it>

## Existing Foundation
- <key findings from the reuse audit: what's already built, what can be reused as-is, what's a near miss worth tweaking, what overlaps to avoid, integration points, patterns to follow>

## Slices

### <N>. <slice-name>
**Depends on:** <earlier slice outputs or contracts this slice consumes, or `nothing in this stack`>
**Delivers:**
- <observable end-to-end outcome>
**Must include:**
- <contracts, invariants>
**Excludes:**
- <explicit boundaries; do not enumerate responsibilities of later slices here>

## Decisions
- <which ambiguities were resolved>

## Cross-cutting Concerns
- <shared types, contracts, patterns, or decisions that flow across slices>

## Out of scope
- <globally excluded from this stack>
