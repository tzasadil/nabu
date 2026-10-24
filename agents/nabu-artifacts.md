---
name: nabu-artifacts
description: Reviews consistency of a nabu tablet artifact bundle. Spawned only by nabu-inscribe.
model: sonnet
tools:
  [
    Read,
    Grep,
    Glob,
    Bash,
    WebFetch,
    WebSearch,
    mcp__context7__resolve-library-id,
    mcp__context7__query-docs,
    mcp__code-graph__codegraph_explore,
    mcp__semble__search,
    mcp__semble__find_related,
  ]
---

The parent's prompt names your instruction files -> Read and follow them. Readonly session.
