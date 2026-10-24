---
name: nabu-learn
description: Mine past nabu sessions into learned rules, reviewer rules, and process improvement candidates, each backed by receipts.
disable-model-invocation: true
---

# Nabu Learn

Learning loop for the `nabu-*` skills. A nabu session is one that loaded a `nabu-*` skill (Skill tool call or slash command). OpenSpec sessions (`openspec-*` skills) stay out. Recurring corrections -> rules, recurring friction -> ranked candidates the user picks from. Every rule and candidate cites its sessions.

**Corrective only.** Every rule, default, and candidate stops a mistake agents repeat, or a question the user keeps answering. A step the user had to prompt counts as a mistake. Never distill what went well into a rule or skill.

## Repo

- The repo is the `jj root` of the working directory. It must hold `skills/nabu-learn`, else stop and tell the user to run from the nabu repo.
- Run scripts and edit files as `<root>/skills/...`, never the installed copy. Edit only files under `<root>/skills/`.

## Files

- `scripts/mine_critiques.py`: mining commands, `parent`, `lint`, `mark`. Run `--help`.
- `last-run.json`: end of the last mining window. Default `--since` of every mining command => sessions under older skill rules stay out.
- [../with-implementer/implementer-rules.md](../with-implementer/implementer-rules.md): rules every implementer and verify lane gets. Candidates only, no write.
- [receipts.md](receipts.md): `Rules`, `Watchlist`, `Candidates`, `Retired` tombstones. Local and gitignored => quoted sessions stay private. Missing -> create it with those four `##` sections.
- [learned-rules.md](../nabu-inscribe/learned-rules.md): **prevention rules** under `Design`, split by owning plan-time reviewer (`Opponent`, `Artifacts`). **Defaults** under `Defaults`. Authors design within all of it. Each reviewer checks only its own heading.
- **Reviewer rules** live in reviewer instruction files:
  - Verify lanes, `nabu-verify/reviewers/*.md`: code-level classes. Lane owners: `Lane ownership` in `nabu-verify/SKILL.md`.
  - Plan-time, `nabu-inscribe/reviewers/{artifacts,opponent}.md` and `nabu-outline/reviewers/opponent.md`: narrowing or `Do not challenge` lines only. Their prevention rules live under their `learned-rules.md` heading.

## Modes

- **Incremental** (default): new signals since `last-run.json`.
- **Audit** (user asks): every mining command with `--all`. Same workflow, plus a review of every existing rule:
  - Promote watchlist entries and `watch` candidates the full history now backs.
  - Receipt fails a disposition re-check -> drop it. Rule left under the bar -> back to the watchlist, or retire it.
  - Process candidates still need the instruction-age check.
  - Skip fragments already quoted in `receipts.md`.

## Workflow

1. **Open the batch** first, as the agent (not the script). Every write of this run lands in one new revision of the nabu repo => the user reviews or abandons it as a whole.

   ```bash
   # use to create the batch revision in the nabu repo
   jj -R <root> new -m "nabu-learn: <date>"
   ```

2. **Mine** into scratch files, then grep and read slices. `findings` alone runs to hundreds of KB.

   ```bash
   # use to dump all four sources; the header line prints since and mined_at
   for c in findings feedback decisions runs; do python3 <root>/skills/nabu-learn/scripts/mine_critiques.py "$c" > "<scratch>/$c.md"; done
   ```

   - Keep the earliest of the four `mined_at` times for step 11.
   - `--all` in audit mode only. `--since` only when the user asks. Honor a user scope (`--project`, `--skill`).
   - All empty -> `mark --at <mined_at>`, report, stop.
3. **Load.** `learned-rules.md`, `receipts.md`, and the reviewer files of every skill in the mined output.
4. **Sort feedback.** Each user message is one of:
   - **Design critique:** challenges approach, scope, contracts, architecture. Feeds rules.
   - **Implementation or UI note:** missing icon, copy, layout. Report-only.
   - **Steering:** `continue`, `build`, `go`. Feeds the Handoff lens when it recurs.
5. **Cluster** findings, design critiques, and decisions by root-cause class, not by file or feature. `debugging-duck` findings first: top-priority source. Each is a free-text report: split it into its claims first. Class = one sentence that holds in any repo: "limits with no observed consumer", not "100-template cap".
6. **Check disposition** of reviewer findings: `parent <ref>` for each class member that could reach a rule. Label each with a receipt-format disposition. Unclear -> drop.
7. **Route** each class (table below).
8. **Write** rules, receipts, and watchlist entries. Draft each rule per [rule-writing.md](rule-writing.md).
9. **Process candidates.** Apply the lenses below. Record each in `## Candidates` as `watch` or `proposed`. Do not wait for picks: finish steps 10-11, then report. User picks in a follow-up turn -> apply in the same voice, same revision, rerun `lint`.
10. **Check.** `lint` until 0 errors. Then, when `fd -L -g validate_skills.py ~/.claude/skills` finds the `writing-skills` script, validate the skills:

    ```bash
    # use to check frontmatter, naming, and links of the edited skills
    uv run -q --no-project --with pyyaml python <found path> <root>/skills
    ```

    `transcript gone` warning is fine: the quote stays the evidence.
11. **Mark.** `mark --at <mined_at>`. Same revision as the rules.

## Routing

| Signal | Write |
| --- | --- |
| Plan-time reviewer (inscribe artifacts/opponent, outline opponent) caught it | Prevention rule under the owner heading |
| `debugging-duck` caught it | Same rows as a verify lane, in any session. Lane = the class's owner |
| User critiqued a design that planning and its reviewers let through | Prevention rule under the owner heading |
| Verify caught it, root cause is a blueprint decision | Prevention rule under the owner heading |
| Verify lane caught a code-level class its lane file does not name | Reviewer rule in that lane file |
| Same answer to the same kind of question in 2+ changes | Default: the agent applies it without asking |
| `custom` or non-recommended answers to the same kind of question in 2+ changes | Default encoding what the user picked => the recommendation changes |
| Answers to the same kind of question differ across changes | Real product choice. No rule |
| Rejected in 2+ distinct changes | Narrow the reviewer rule that produced it, or add a `Do not challenge` line |
| Class needs one repo's framework, module, or domain | Reviewer judges it -> report as `rule-forge` `coding-standard` candidate. Author needs it while writing -> report as `AGENTS.md` candidate. No write |
| Implementation-only mistake a verify lane already names | Report as `nabu-build` + `nabu-tweak` candidate. Holds in any repo -> `implementer-rules.md` candidate. One repo -> `AGENTS.md` candidate. No write |

One class can hit several rows. Write each.

Owner heading, by what the rule asks of the design:

- `Opponent`: drop or simplify (machinery, layers, scope), or verify a premise.
- `Artifacts`: add the missing piece (test, failure outcome, recorded decision).

## Process lenses

Candidate needs the pattern in 2+ sessions. `resumes:` transcript in `runs` = same session as its original. One session -> `watch`.

Instruction age: before proposing an instruction change, find when the current wording landed. Walk `jj evolog -r <rev>` in the nabu repo, take the first snapshot whose `jj file show` contains the line, in UTC. `jj file annotate` shows the last rewrite, not first appearance. Count only sessions that loaded the skill after that. Older signals are already fixed.

- **Skill cleanup:** a loaded `nabu-*` skill let the agent go wrong the same way twice, or the user re-steered the same way. Name the ambiguous, contradictory, or dead instruction. Check siblings that run the same step (`nabu-build` + `nabu-tweak`, propose + outline opponents): fix holds with the skill's name removed -> one candidate per sibling.
- **Handoff:** `runs` shows `(user)` or `(prompted)` where the previous skill should have handed off ("build" typed after inscribe). Name the failed handoff instruction.
- **Automated checks:** a class a deterministic check could catch: lint, types, a scan in `nabu-verify/scripts/`. Read the repo's check command first. Existing check unwired or silently broken -> that is the finding. Before proposing a new check, run it over the mined sessions or repo. Record hits, real defects among them, and what silencing the rest takes.
- **Tool economy:** one skill's tool count or input far above the same skill elsewhere, or one tool dominating. Name the cheaper path.
- **Information access:** the user supplied facts the agent could not reach (logs, database state, third-party systems). Name the read-only access.
- **Navigation:** sessions that search long for the same files. Name the pointer for that repo's `AGENTS.md`.
- **No-ops:** instruction restating the model default, user rules, or another skill. Silence alone is no proof: a working prevention rule is also silent.
- **Bad description:** a `nabu-*` skill that fit and never loaded (requirements changed during build, no `nabu-tweak`), loaded and changed nothing, or loaded on unrelated work. Fix the description, not the body.
- **Skill merge or new skill:** skills that always load together, or the same instruction or correction across sessions.

Candidate ledger:

- Read `## Candidates` before reporting. `watch`, `proposed`, `declined` -> report again only with evidence after its status date (audit mode: full history counts), and say it was raised before. `applied` + pattern recurs after its date => failed fix. Report a check or hook that enforces it. Rewrite only when the applied wording was unclear.
- User picks it -> `applied`. User rejects it, or already fixed -> `declined` + reason. No answer -> `proposed`.

## Invariants

- **Two changes.** Rule needs 2+ distinct changes of one polarity: accepted signals, or rejected ones for a `Do not challenge` line. Sources mix (one reviewer finding + one user critique counts). Watchlist sightings count. Slices of one stacked tablet = one change. One sighting -> watchlist.
- **Receipt first.** No receipt -> no rule. Fragments quote the source verbatim, trimmed with `…`: reviewer problem text, user message, or question + answer.
- **Existing coverage wins.** Class already has a rule -> append fragments, never restate. Fragment dated after `Last edited` => the owner had the line and missed it => rewording rarely helps. First that fits, set `Last edited` on any text change:
  - Line unclear -> rewrite it.
  - Wrong owner -> move it to the other heading.
  - Check can catch it -> propose that check (Automated checks lens).
  - Neither -> keep the line unchanged.
- **Reconcile.** Before writing a rule, read the rules file and reviewer files for statements it overrides. Rewrite or scope the older statement in the same batch. Conflict with a repo's `AGENTS.md` -> scope the new rule (the repo wins), note a `Counter:` line in the receipt.
- **Portable.** Rules hold in any repo. No project, module, or product names.
- **Artifact altitude.** Rules speak `stack.md` and `blueprint.md` vocabulary: scope, components, contracts, failure outcomes, tests, decisions. No code shapes (class kinds, helpers, state variables, concurrency mechanics). Class visible only in code -> verify-lane reviewer rule, or rule-forge `coding-standard` candidate when it holds in one repo only.
- **Rule voice.** One line: target state, plus why when not obvious. End with the ID: `(L7)`.
- **Short file.** `learned-rules.md` loads on every outline and inscribe run. Past ~30 rules -> merge into broader classes before adding.
- **Placement.** Reviewer rules under the matching existing heading. New heading only when none fits.
- **Write scope.** Without asking: `nabu-inscribe/learned-rules.md`, rule lines in reviewer instruction files, `receipts.md`.
  - Other files under `<root>/skills/` (skill bodies, descriptions, `with-implementer/implementer-rules.md`): only after the user picks the candidate.
  - Candidates for `AGENTS.md`, `CODING_STANDARDS.md`, checks, or `rule-forge`: report, never write. Never `~/.claude/CLAUDE.md`.

## Receipt format

```md
### L7: <rule tldr>
- Target: `nabu-inscribe/learned-rules.md` :: <owner heading: Opponent | Artifacts | Defaults>
- Class: <root-cause class, one sentence>
- Introduced: 2026-09-30
- Last edited: 2026-10-05
- Evidence:
  - 2026-09-28 · <project> · `<change>` · inscribe/opponent · `cc:<project>/<session>/<agent>` · adopted
    > <verbatim fragment>
  - 2026-09-30 · <project> · `<change>` · user during verify · `cc:<project>/<session>` · user
    > <verbatim fragment>
```

- `Introduced`: date the entry entered its current section. Promotion from the watchlist resets it.
- `Last edited`: last change to rule text, target, or class. Appended evidence does not count: it carries its own dates.
- Dispositions. Accepted: `adopted`, `fixed`, `deferred` (as valid), `user` (critique or answer). Rejected: `rejected`. Candidate process evidence: `observed` (agent's own text or tool behavior), `prompted` (user had to trigger a step).
- One `Target` per entry.

Candidate format, under `## Candidates`. Status: `watch`, `proposed`, `applied`, or `declined`:

```md
### C3: <candidate tldr>
- Lens: <process lens>
- Target: `nabu-inscribe/refine.md` :: <heading>
- Status: proposed · 2026-10-03
- Change: <the proposed edit, or what was applied, or why declined>
- Evidence:
  - 2026-10-01 · <project> · `<change>` · runs · `cc:<project>/<session>` · prompted
    > <verbatim fragment>
```

IDs increase, never reused. Watchlist entries: rule shape without an ID. Promotion moves the entry to `Rules` with the next `L` ID. Retiring a rule, or moving it back to the watchlist, leaves `### L<n>: <tldr>` + reason under `## Retired` => the ID stays taken.

## Insights

- Opponent findings and user critiques feed most prevention rules. Over-engineering classes recur most.
- End-of-run design critiques cluster in `during verify` and `during tweak`. `agent:` shows what the user reacted to.
- Most answers take the recommended option. No signal alone. A question recurring across changes is.
- Verify parents list `REJECT` / `FIX` / `DEFERRED` / `ASK` per finding in their final report. Inscribe parents keep dispositions internal: look for blueprint edits, pushback text, user replies. `ASK` counts only after the user answered.
- Clustering on `problem`, `user`, and `q` lines first is fastest. Read `evidence` and `parent` only for ambiguous members.

## Report

- Sessions, findings, user messages, and answers mined, with the date range.
- Rules and defaults added per file, with IDs and receipt counts. Fragments appended to existing receipts.
- Watchlist additions and promotions.
- Candidates ranked by severity: process lenses, reviewer noise, `rule-forge`, `AGENTS.md`, `CODING_STANDARDS.md`, check candidates, `nabu-build` + `nabu-tweak`, implementation notes. Each with lens, evidence refs, proposed edit.
