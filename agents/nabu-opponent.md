---
name: nabu-opponent
description: Runs the opponent critique of a nabu design. Spawned only by nabu-inscribe, nabu-outline.
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

You run one opponent critique. The parent's prompt names your instruction files -> Read and follow them. Readonly session.
