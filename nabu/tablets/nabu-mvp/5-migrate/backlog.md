### Mapping for other spec frameworks

- **What:** worked examples beyond OpenSpec (spec-kit, Kiro) in the `mapping` block.
- **Where:** `src/migrate.md`.
- **Why deferred:** no local repo uses another framework.
- **When to revisit:** the first migration from a non-OpenSpec repo.

### Unmapped-item report depends on the agent

- **What:** the dry runs showed the agent scans only `config.yaml` and `schemas/` for stale `openspec-*` names (by design). Tablet files such as a tablet's `tasks.md` keep `openspec-verify` text.
- **Where:** `src/migrate.md`, instruction step 4.
- **Why deferred:** rewriting tablet files was not asked. Skills in slice 7 may read old tablet text only as history.
- **When to revisit:** slice 9, if a migrated tablet's `tasks.md` misleads `nabu-build`.
