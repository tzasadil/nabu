# Blueprint nabu-mvp/7-execution-skills

## Feature description

The execution skills and their hard dependencies move into this repo and work on tablets through `nabu`:

- `skills/nabu-build`: implement a plain tablet or one slice through `with-implementer`.
- `skills/nabu-verify`: review an implementation against its tablet, fix, sync artifacts.
- `skills/nabu-archive`: settle the backlog, then `nabu archive`.
- `skills/nabu-learn`: mine nabu sessions into architecture, reviewer, and implementer rules.
- `skills/with-implementer`: delegate implementation to the `implementer` agent with the pin -> diff pattern, including its instruction file `implementer.md` and `implementer-rules.md`. Name kept, it is generic.
- `agents/nabu-reviewer.md`, `agents/implementer.md`.

It also closes the slice 6 refine handoff gap and renames `openspec-verify` in the built-in text. Sources are `/data/src/wiki/skills/{openspec-apply,openspec-verify,openspec-archive,openspec-learn,with-implementer}/` and `/data/src/wiki/agents/openspec-reviewer.md`. The old skills stay installed until slice 9.

## Agent Decisions

- **Port, not slim.** Same rule as slice 6: behavior and wording stay, only OpenSpec coupling changes. Calls become `nabu status`, `nabu list`, `nabu instructions build` (`read:` replaces `contextFiles`), `nabu archive`. Paths become `nabu/tablets/...`. The `openspec/**` exclusion in the verify scan scripts becomes `nabu/**`. Other rename sites: `shared-reviewer-rules.md` path notes, `diff-scan` help, the `with-implementer` brief (`openspec change dir` -> tablet or slice folder), `implementer.md` mode text (`plan (OpenSpec change)` -> tablet). `skills/nabu-verify/implementer-rules.md` stays a relative symlink to `../with-implementer/implementer-rules.md` => one rules file.
- **Repo root and CLI failures.** Same rules as slice 6: run `nabu` and write from the repo root, stop on a non-zero exit and show `error:` and `fix:`.
- **`implementer.md` source.** The installed `~/.claude/agents/implementer.md` wins over the wiki copy: it is the one that runs, and it is a superset (also disallows `ReportFindings`).
- **Archive flow.** `nabu-archive` takes whole tablets only. A slice target -> it names the tablet and asks. Per tablet: `nabu status <tablet>`; not `ready to archive` -> ask before going on. Then settle each backlog item, in the tablet's `backlog.md` or in every slice's `backlog.md`: keep (stays in the archive), move (to a new tablet), or drop. One approval for the whole set, as the old skill did. Then `nabu archive <tablet>` first. Only for a tablet that archived, the moves run (`nabu new <new-tablet>`, item text written into its `backlog.md`) => a failed archive never duplicates items. A failed tablet is reported with its moves skipped and the rest continue. It never moves files itself.
- **Learn scope.** A nabu session is one that loaded a `nabu-*` skill (Skill tool call or slash command). `mine_critiques.py` builds the name alternation at runtime from the `nabu-*` folders under its skills root => a tablet named `nabu-mvp` never matches, and new skills join without a regex edit. Learn edits only files under `skills/` of this repo. Candidates for `AGENTS.md`, `CODING_STANDARDS.md`, checks, or `rule-forge` are reported, never written. The repo is the `jj root` of the working directory, which must hold `skills/nabu-learn`, else learn stops and says to run from the nabu repo. Scripts and edits then use `<root>/skills/...`, never the installed copy. Required check: `mine_critiques.py lint`. `validate_skills.py` runs when the wiki is present. "In the wiki" becomes "in this repo".
- **Receipts move and their targets are rewritten.** `receipts.md` and `last-run.json` move as history. Receipt `Target:` lines are rewritten mechanically (`openspec-<x>/` -> `nabu-<x>/`). `mine_critiques.py` sets `RULES_PATH` to `nabu-inscribe/learned-rules.md` and its lint glob covers `nabu-*/**/*.md` and `with-implementer/**/*.md` => `lint` stays green.
- **Rule voice.** `skills/nabu-learn/rule-writing.md` replaces every `writing-skills` reference in learn. It holds the parts of `writing-skills` that shape rule text: writing style, no speculative guardrails, positive target, pruning, sprinkling, restated globals. Learn links it at the step where it drafts a rule => learn works on a machine without the wiki.
- **Optional tools.** `debugging-duck` findings, the `renaming`, `rule-forge`, and `diagnosing-bug` skills, and the wiki's `validate_skills.py` are used when installed. Their absence never blocks a skill.
- **Handoffs.** `nabu-inscribe` refine continues to `/nabu-build`. `nabu-tweak` is unchanged: its source never auto-runs verify. Apply hands off to verify, verify to the next slice's propose or to archive.
- **Built-in text.** `defaults/schemas/specless/templates/tasks.md` names `nabu-verify`. The built-in apply instruction names no skill, so it needs no edit. Snapshots that print the template change accordingly. Verify's Phase 4 tick matches the template wording (`Use skill \`nabu-verify\``).
- **Binary refresh.** Rerun `cargo install --path .` after the `defaults/` change.

## User Decisions

- Skills live in this repo, named `nabu-*`. `with-implementer` and `implementer` keep their names. Port now, slim in the next stack.
- Hard dependencies ship with nabu. `debugging-duck`, `renaming`, `rule-forge` stay in the wiki, optional.
- `nabu-learn` touches only nabu skills and sessions. Rule-writing guidance comes from `writing-skills`, copied.
- #1907 lives in `nabu-archive`: settle the backlog before the move.

## Flow

- `nabu-build <target>` -> `nabu instructions build <target>` -> read `read:` files -> `/with-implementer` (mode `plan`, review `deferred`) -> `/nabu-verify <target>`.
- `nabu-verify <target>` -> pin -> spawn lanes (`nabu-reviewer`) -> triage -> fixes via `/with-implementer` -> sync artifacts (stack edits squashed with `--keep-emptied`) -> next slice or archive.
- `nabu-archive <tablet>` -> `nabu status` -> settle backlog -> `nabu archive <tablet>` -> moves for archived tablets.
- `nabu-learn` -> mine nabu sessions -> rules in `skills/nabu-inscribe/learned-rules.md`, reviewer files, `skills/with-implementer/implementer-rules.md` -> receipts.

## Contracts

- **No OpenSpec coupling.** `rg -in 'openspec|skip_specs' skills agents defaults` matches only: quoted session text inside `skills/nabu-learn/receipts.md`, the phrase "past OpenSpec sessions" in `learned-rules.md`, and one sentence in `skills/nabu-learn/SKILL.md` that excludes OpenSpec sessions.
- **No template text.** `rg -n '^# (Stack:|Blueprint|TODO Checklist)' skills` finds nothing.
- **Agents.** `nabu-reviewer` keeps the tools and role of the wiki `openspec-reviewer`, only `name` and `description` change. `agents/implementer.md` equals the installed copy.
- **Handoffs.** Every `/<name>` or `` `<name>` `` skill reference in `skills/` names a folder in `skills/` or one of `debugging-duck`, `renaming`, `rule-forge`, `diagnosing-bug`. Refine continues to `/nabu-build`.
- **Learn tooling.** `mine_critiques.py lint` exits 0 on the ported tree. One mining run over the sessions of this repo reports nabu skill phases only (no `mvp`, no `openspec-*`). `rule-writing.md` is linked from the drafting step, and `writing-skills` appears nowhere in `skills/nabu-learn` except as the copy's source note.
- **Scripts.** From `/data/src/specless`: `mine_critiques.py --help` exits 0; `skills/nabu-verify/scripts/diff-scan` on the slice 6 revision exits 0 and lists no `nabu/**` path; `skills/nabu-verify/scripts/scan-utility-handrolls.sh` on the same revision exits 0.
- **Validation.** `validate_skills.py` on `skills/` checks 8 skills with 0 errors.
- **Built-in text.** `nabu instructions tasks <target>` prints `nabu-verify` in its template block.

## Out of scope

- Installing skills globally (`nabu init`, slice 8). Removing the old skills (slice 9).
- Slimming `nabu-verify` or any skill (next stack).
- Mining old OpenSpec sessions.

## Gotchas

- `with-implementer` exists twice until slice 9: wiki (global) and this repo (project). In Claude Code the personal (global) copy wins on a name clash, so sessions here run the wiki copy until slice 9 removes it.
- Learn ledger split: until slice 9 existing repos still run `openspec-learn`, which writes the wiki's rule files and `receipts.md`. Slice 9 merges any newer wiki rules and receipts into `skills/` (renumbering colliding IDs) before it deletes the wiki copies.
- The completion gate of this slice still runs the old `openspec-verify`: the new one lands with this slice and is reviewed by the old one.
- `nabu-learn` reads `skills/nabu-inscribe/learned-rules.md` and `skills/with-implementer/implementer-rules.md` by relative path. Keep all skills side by side under `skills/`.
