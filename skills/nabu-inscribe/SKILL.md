---
name: nabu-inscribe
description: Use when turning an approved design or clear request into a new tablet with an implementation-ready plan, or when refining proposed tablet artifacts before implementation.
---

# Nabu Inscribe

Refine-only request -> select the target, then skip to [refine.md](refine.md).

A non-zero `nabu` exit stops this skill. Show the user its `error:` and `fix:` lines. Never continue on empty output.

Run `nabu` and write artifacts from the repo root: its paths are root-relative. Prefix the root to pass reviewers absolute paths.

## Workflow

1. **Resolve the target**
   - Derive a kebab-case name from a clear description.
   - Continue an explicitly named, pre-scaffolded stack slice from an earlier proposal cycle.
   - Any other existing name (`nabu list`) -> ask whether to continue it or create another.
   - Stack mode: read `stack.md`. Confirm the target (`<tablet>/<N>`) is a declared slice.
   - jj: edit the target slice's unique pre-created revision before writing artifacts. Stop and ask when no unique revision matches.
   - jj, plain tablet: `@` empty -> `jj describe -m "<name>"` and use it. `@` holds more than about 20 changed lines -> `jj new -m "<name>"` first.
2. **Scaffold**
   - New plain tablet: `nabu new "<name>"`
   - Stack slices already exist from `nabu-outline`. Target the explicitly named one.
   - Declared slice with no folder (`nabu status` error) -> scaffold every missing slice in N order. jj: `jj edit <stack-rev>`, `nabu new <tablet>/<slice>`, then `jj new --insert-after <slice N-1 revision, or stack-rev for N=1> -m "<tablet> <N>: <slice>"`. Then edit the target revision.
   - `stack.md` is the cross-slice source of truth. jj: it lives in the stack revision. Edit it there, or move a slice-side edit with `jj squash -u --keep-emptied --from @ --to <stack-rev> <path>`. Without `--keep-emptied` an emptied slice revision is abandoned.
3. **Audit reuse**
   - Standalone: inspect code, `AGENTS.md`, docs, and history yourself (no subagent) for reusable code, near misses, integration points, patterns, conflicts, and real gaps.
   - Skip in stack mode. `nabu-outline` already supplied it.
4. **Grill the user**
   - relentlessly ask until every material ambiguity is resolved. For each big decision, offer two or three viable approaches and mark one as recommended. `Defaults` in `learned-rules.md` settle recurring questions.
   - Ask only for material product decisions the artifacts must encode: purpose, constraints, success, and contract-shaping choices. Make routine implementation decisions yourself.
   - An unambiguous or already approved request needs no questions.

5. **Build artifacts**
   - Build order: `nabu status <target>`
   - For each artifact with satisfied dependencies:

     ```bash
     # use to fetch the template and write path for one artifact
     nabu instructions <artifact-id> <target>
     ```

   - Read the `read:` files, then write the `<template>` to the `write:` path.
   - Apply `<context>`, `<rules>`, and `<instruction>` as guidance. Never copy their wrapper tags into the artifact.
   - Stop when `nabu instructions build <target>` reports `state: ready`. An artifact outside that closure (`backlog`) is written only when there is deferred work, as its instruction says.

## Artifact intent

- Scope stack artifacts to the current slice's declared responsibility so later slices remain independent.
- Contracts in the blueprint, deferred work in the backlog, executable work in tasks.
- Local, loosely coupled design. Easy deletion > speculative extensibility.
- Load-bearing external claims (API, config key, version support, vendor limit): check official docs (context7 or web) before the blueprint relies on them. Memory-only claim -> mark it `UNCERTAIN` in the blueprint.
- Blueprint stays at outcome, contract, and component level. Code shapes (classes, helpers, state variables) are build's call unless a contract depends on one.
- Tests protect boundary behavior so implementation details remain replaceable.

## Handoff

Follow [refine.md](refine.md) after artifacts are ready. Generated artifacts need an independent consistency pass. Refine only the tablet or slice proposed in this cycle.
