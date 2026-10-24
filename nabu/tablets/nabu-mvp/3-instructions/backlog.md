### Archive-phase guidance

- **What:** `nabu instructions archive <target>` serving config guidance for the archive step (upstream #1913).
- **Where:** `instructions` subcommands, config `[rules]` keys.
- **Why deferred:** `nabu-archive` (slice 7) reads the backlog and settles it itself. No archive guidance exists in any config today.
- **When to revisit:** a repo config grows rules meant only for archiving.

### Artifact ids that shadow subcommands

- **What:** reject artifact ids `apply` and `stack`, which `nabu instructions` treats as subcommand words.
- **Where:** `schema::resolve` in `src/schema.rs`.
- **Why deferred:** no schema uses these ids. A guard for an unobserved case.
- **When to revisit:** a schema author defines such an artifact and cannot reach its instructions.

### Phase rules for apply and stack

- **What:** `[rules] apply` and `[rules] stack` keys served in the apply and stack prompts.
- **Where:** rules-key check, apply and stack prompts.
- **Why deferred:** no existing config sets rules for these phases.
- **When to revisit:** a config grows rules meant only for apply or for writing the stack.
