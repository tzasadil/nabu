# Blueprint nabu-mvp/6-planning-skills

## Feature description

The planning skills move into this repo and work on tablets through `nabu`:

- `skills/nabu-outline`: design a stacked tablet and its slices.
- `skills/nabu-inscribe`: write a plain tablet's or one slice's artifacts, then refine them.
- `skills/nabu-tweak`: change requirements of a tablet already in implementation.
- `agents/nabu-opponent.md`, `agents/nabu-artifacts.md`: the reviewer agents they spawn.

Sources are `/data/src/wiki/skills/openspec-{outline,propose,tweak}/` (reviewers included) and `/data/src/wiki/agents/{openspec-opponent,artifacts}.md`. The old skills stay installed until slice 9.

## Agent Decisions

- **Bare `nabu` on PATH.** Skills call `nabu`. This slice installs it with `cargo install --path .` into `~/.cargo/bin` (on PATH). Undo: `cargo uninstall nabu`. The install is a snapshot: rerun it after any CLI or `defaults/` change (slices 7 and 8 carry this in their gates).
- **Refine stops at ready until slice 7.** `nabu-build` does not exist yet, so refine ends with "ready to apply, stop". Slice 7 switches the handoff to `/nabu-build` => no dead control in between.
- **Stop rule for propose.** Build artifacts in `nabu status <target>` order until `nabu instructions build <target>` reports `state: ready`. An artifact outside the apply closure (the built-in `backlog`) is written only when there is deferred work, as the schema instruction says.
- **Plain tablets.** `nabu-inscribe` runs `nabu new <tablet>` for a new plain tablet. Slices already exist from outline.
- **CLI failures.** A non-zero `nabu` exit stops the skill and shows the `error:` and `fix:` lines to the user. No step continues on empty output.
- **Stack revision first.** Before its first write, outline picks the stack revision: `@` empty -> `jj describe -m "<tablet>: stack"`, else `jj new -m "<tablet>: stack"`. Then `nabu new <tablet> --stack` and `stack.md` from `nabu instructions stack <tablet>` (the tablet must exist first). After approval: `nabu new <tablet>/<slice>` per slice in N order matching the `### <N>.` headings, then one `jj new -m "<tablet> <N>: <slice>"` per slice. The stack revision holds `stack.md` and every marker, slice revisions start empty.
- **Edits to `stack.md` from a slice revision** move with `jj squash -u --keep-emptied --from @ --to <stack-rev> <path>`. Without `--keep-emptied` an emptied slice revision is abandoned (observed while building this stack).
- **Bundle check for nabu.** `check_bundle.py` finds the root by `nabu/`, treats `nabu` as a command needing a working directory, and checks one folder: a plain tablet or one slice. A folder without `tasks.md` exits 2 with an error => the gate never passes on empty input.
- **Project links for dogfooding.** `.agents/skills -> ../skills`, `.claude/skills -> ../.agents/skills`, `.claude/agents -> ../agents`. The third link is new against `stack.md` and is recorded there.
- **Skill edits follow `writing-skills`.** Validate with its `validate_skills.py` over `skills/`.

## User Decisions

- Skills live in this repo, named `nabu-*`. Port now, slim in the next stack: behavior and wording stay except for OpenSpec coupling.
- Schema knowledge and templates come from nabu: no artifact list and no template text in skills. `stack.md` comes from `nabu instructions stack`.
- Architecture rules stay with the skills: `skills/nabu-inscribe/learned-rules.md` unchanged, including its `## Defaults` section. Rule-like prose inside skills (for example "Artifact intent") stays too: it is skill content, not repo config.
- Hard dependencies ship with nabu. `with-implementer` and `implementer` arrive in slice 7. `debugging-duck`, `renaming`, `rule-forge` stay in the wiki and are optional.

## Flow

- `nabu-outline` -> audit -> grill -> design -> stack revision -> `nabu new <tablet> --stack` -> write `stack.md` from `nabu instructions stack` -> opponent (`nabu-opponent`) -> approval -> slice markers + jj chain -> `/nabu-inscribe <tablet>/1`.
- `nabu-inscribe <target>` -> plain and new: `nabu new <tablet>` -> `nabu status <target>` -> per ready artifact: `nabu instructions <artifact> <target>` -> write -> repeat until apply is `ready` -> refine (`check_bundle.py`, `nabu-artifacts`, `nabu-opponent`) -> report ready and stop (slice 7: `/nabu-build <target>`).
- `nabu-tweak <target>` -> `nabu status` + `nabu instructions build` (its `read:` line replaces `contextFiles`) -> edit artifacts -> `/with-implementer`.

## Contracts

- **No OpenSpec coupling.** `rg -in 'openspec|skip_specs' skills/nabu-outline skills/nabu-inscribe skills/nabu-tweak agents` matches only this allowlist: the phrase "past OpenSpec sessions" in `learned-rules.md`. Anything else is a defect.
- **No template text.** No file under `skills/nabu-*` contains a heading line of a built-in template (`# Stack:`, `# Blueprint`, `# TODO Checklist`).
- **CLI usage.** Every `nabu` call in the skills exists and matches the slice 1 to 3 contracts.
- **Agents.** `nabu-opponent` and `nabu-artifacts` keep the tools, model, and read-only role of their wiki sources. Only `name` and `description` change.
- **Bundle check.** Per the `Bundle check for nabu` decision. A clean folder exits 0, a folder with an unresolved path or an uncertainty marker outside Gotchas exits 1.
- **Stack in jj.** After the outline sequence, also when `@` held unrelated edits at the start: the stack revision holds `stack.md` and every slice marker, the unrelated edits stay in their own revision, and each slice revision exists and is empty.
- **Validation.** `validate_skills.py` on `skills/` checks 3 skills with 0 errors.
- **Links.** `.claude/skills/nabu-outline/SKILL.md` and `.claude/agents/nabu-opponent.md` resolve.

## Out of scope

- `nabu-build`, `nabu-verify`, `nabu-archive`, `nabu-learn`, `with-implementer`, `implementer`, `nabu-reviewer` (slice 7). The pin -> diff pattern lives there.
- Installing skills globally (`nabu init`, slice 8). Removing the old skills (slice 9).
- Slimming or rewording skills beyond the coupling.

## Gotchas

- Until slice 7 lands, `nabu-tweak` delegates to the wiki's `with-implementer`, whose text still names OpenSpec.
- `learned-rules.md` is read by outline and the reviewers through relative paths (`../nabu-inscribe/learned-rules.md`). Keep the skills side by side under `skills/`.
- UNCERTAIN: Claude Code loading agents through a symlinked project `.claude/agents` folder. Skills through the symlinked `.claude/skills` are confirmed: the session that built this slice listed `nabu-outline`, `nabu-inscribe`, `nabu-tweak`. Settled by task 2.5.
- The completion gate's `openspec-verify` Phase 4 squash names `openspec/stacks/...` and lacks `--keep-emptied`. In this repo use `nabu/tablets/nabu-mvp/stack.md` and `--keep-emptied`.
