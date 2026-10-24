### Optional artifacts

- **What:** a schema flag that marks an artifact optional, so `status` shows it as `optional` instead of `ready` forever (built-in `backlog`).
- **Where:** schema `[[artifacts]]` keys, artifact state.
- **Why deferred:** `ready` on an unwritten backlog is harmless. No observed confusion.
- **When to revisit:** a skill or the user treats an unwritten backlog as missing work.

### `generates` as a plain file name

- **What:** reject `generates` values with `/`, `\`, or a leading `.` in `schema::resolve`, like `stack`.
- **Where:** `src/schema.rs` graph checks.
- **Why deferred:** slice 2 only reads files. The guard protects writers.
- **When to revisit:** slice 3 prints `generates` as an output path that skills write to.
