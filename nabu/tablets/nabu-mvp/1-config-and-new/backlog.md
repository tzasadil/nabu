### Schema choice per call

- **What:** `nabu new <tablet> --schema <name>` to pick a schema without editing config.
- **Where:** `nabu new` command.
- **Why deferred:** the config `schema` key covers the one-user case. No observed need.
- **When to revisit:** a repo starts using two schemas side by side.

### Relaxed name rule

- **What:** allow names outside kebab-case (upstream #836 reports strict names blocking namespacing).
- **Where:** tablet and slice name check.
- **Why deferred:** every existing change name is kebab-case.
- **When to revisit:** migration (slice 5) meets a name the rule rejects.

### Unknown-schema fix names the wrong config file

- **What:** when the configured schema is unknown, the fix always names `nabu/config.toml`, even when the global `config.toml` set the name.
- **Where:** `src/config.rs` (`Config.schema` does not record its source layer), `src/new.rs::new_tablet`.
- **Why deferred:** needs the source file tracked per merged key. Rare: only a typo in the global schema name triggers it.
- **When to revisit:** slice 2 or 3 adds another merged key whose errors need the source file.
