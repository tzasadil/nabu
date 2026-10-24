### Skill slimming

- **What:** slim `nabu-verify` (5 lanes, about 1300 lines) and make all skills more generic.
- **Where:** `skills/nabu-verify`, other `skills/nabu-*`.
- **Why deferred:** user scoped this stack to a port.
- **When to revisit:** the stack after `nabu-mvp`.

### Regex guard for mine_critiques.py

- **What:** parametrized test that `nabu-mvp/7` and `nabu-build-foo` do not match `SKILL_RE`, and `nabu-verify/reviewers/x.md` and `/nabu-learn` do.
- **Where:** `skills/nabu-learn/scripts/mine_critiques.py`.
- **Why deferred:** no Python test runner in the repo.
- **When to revisit:** slice 8 or the next stack adds a Python check.
