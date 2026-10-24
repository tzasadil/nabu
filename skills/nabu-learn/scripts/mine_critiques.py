#!/usr/bin/env python3
"""Extract nabu learning signals from Claude Code session logs.

Why: reviewers, the user, and questionnaires produce the same corrections run
after run. This script is the raw input for nabu-learn. It only extracts.
Clustering, disposition checks, and rule writing stay with the agent.

Source: ~/.claude/projects/<project>/<session>.jsonl and
<session>/subagents/agent-*.jsonl. Override the root with CLAUDE_PROJECTS.

Commands:
  findings   Reviewer findings. Only a subagent's last output (text or
             SubagentHandback message) counts as its report, so prompts that
             quote the output format never parse. Plus every
             `debugging-duck` report, one free-text finding each, labeled
             with the nabu skill active when it ran (`none`).
  feedback   The user's own messages after a nabu skill loaded, tagged
             with the active skill and the agent text they reply to.
             End-of-run design critiques live here.
  decisions  AskUserQuestion questions with options, the user's answer, and
             whether the answer took the recommended option.
  runs       One block per session: skill flow with who started each skill,
             tool calls and tokens per skill, and signal counts.
  parent <ref>
             Parent activity after one reviewer report arrived: assistant
             text, user text, and edits. Use it to judge whether the parent
             fixed, rejected, or asked about each finding.

  mark [--at ISO]
             Overwrite last-run.json. Pass the mined_at time the mining
             commands printed. Run it at the end of every nabu-learn run.
  lint       Check rule files against receipts.md: every rule has a receipt
             and the reverse, 2+ distinct changes per rule, valid dates and
             statuses, unique IDs, rules-file budget. Exit 1 on errors.

findings, feedback, decisions, and runs take:
  --since ISO  --all  --project SUBSTR  --skill NAME  --json
Without --since or --all they read `last_run` from last-run.json, so a run
skips sessions an earlier run already mined under older skill rules.
"""

from __future__ import annotations

import argparse
import collections
import datetime
import html
import json
import os
import pathlib
import re
import signal
import sys
from dataclasses import asdict, dataclass, field

# Default --since. Overwritten by `mark` after each nabu-learn run.
STATE_PATH = pathlib.Path(__file__).resolve().parent.parent / "last-run.json"
SKILLS_ROOT = STATE_PATH.parent.parent
# nabu-learn itself mines sessions. Its own runs are no nabu phase.
IGNORED_PHASES = {"learn"}

CC_ROOT = pathlib.Path(
    os.environ.get("CLAUDE_PROJECTS") or pathlib.Path.home() / ".claude" / "projects"
)

# Skill phase from a skill name, reviewer prompt, or slash command.
# Built from the skill folders => a tablet named `nabu-mvp` never matches, and a new skill
# needs no edit here.
SKILL_NAMES = sorted(
    p.name.removeprefix("nabu-") for p in SKILLS_ROOT.glob("nabu-*") if (p / "SKILL.md").is_file()
)
SKILL_ALT = "|".join(map(re.escape, SKILL_NAMES)) or "(?!)"
SKILL_RE = re.compile(rf"nabu-({SKILL_ALT})(?![\w-])")
COMMAND_RE = re.compile(rf"<command-name>/?nabu-({SKILL_ALT})</command-name>")
COMMAND_ARGS_RE = re.compile(r"<command-args>(.*?)</command-args>", re.S)
# Tablet under work. Counting distinct changes needs it.
PLACEHOLDER_CHANGES = {"change", "target", "name"}
# Group order = preference.
CHANGE_RE = re.compile(
    r"nabu/tablets/([\w-]+)|Input context: *([\w-]+)"
)
# Skill args usually start with the target, `<tablet>[/<slice>]`: "todo-reminders: add a button".
KEBAB_ARG_RE = re.compile(r"\s*([a-z0-9]+(?:-[a-z0-9]+)+)\b")

# inscribe/outline reviewers emit `<critique>` XML. Older verify runs used XML
# without a status attribute. Current verify reviewers emit `# critique:` Markdown.
XML_HEAD_RE = re.compile(r'<critique agent="([\w-]+)"(?: status="([\w-]+)")?')
XML_FINDING_RE = re.compile(r"<finding>(.*?)</finding>", re.S)
XML_TAG_RE = re.compile(r"<(severity|problem|reference|fix|alternative)>(.*?)</\1>", re.S)
MD_HEAD_RE = re.compile(r"^# critique: *([\w-]*)", re.M)
MD_STATUS_RE = re.compile(r"^status: *([\w-]+)", re.M)
MD_FINDING_RE = re.compile(r"^## *([\w-]+): *(.+)$", re.M)
MD_FIELD_RE = re.compile(r"^- (loc|spec|evidence|hint): *(.*)$", re.M)

# Free-text second-opinion subagent. Its reports never parse as a critique.
DUCK_AGENT = "debugging-duck"

# Harness wrappers inside user messages. None of them is the user speaking.
NOISE_RE = re.compile(
    r"<(ide_opened_file|system-reminder|local-command-stdout|local-command-caveat)>.*?</\1>", re.S
)
SELECTION_RE = re.compile(r"<ide_selection>(.*?)</ide_selection>", re.S)
NOT_HUMAN_PREFIXES = (
    "<task-notification>",
    "[Request interrupted",
    "<command-",
    "<local-command",
    "Base directory for this skill",
    "Another Claude session sent a message",
    "<bash-input>",
    "<bash-stdout>",
    "<bash-stderr>",
)
NOTIFICATION_RE = re.compile(r"^<task-notification>.*?<summary>(.*?)</summary>", re.S)

# Output caps. A verify parent can run for hours after the critique.
PARENT_CHAR_BUDGET = 12000
EDIT_SNIPPET_CHARS = 200
REPLY_CONTEXT_CHARS = 500
# A queued mid-turn message and its echo land seconds apart. A repeat after
# that is a real second message (`build`, `go`) and counts.
DUPLICATE_WINDOW_S = 10
SELECTION_CHARS = 300
MESSAGE_CHARS = 3000


@dataclass
class Finding:
    severity: str
    problem: str
    reference: str = ""
    fix: str = ""
    evidence: str = ""


@dataclass
class Report:
    ref: str
    time: str
    project: str
    skill: str
    reviewer: str
    status: str
    change: str
    findings: list[Finding] = field(default_factory=list)


@dataclass
class Feedback:
    """One message the user typed while a nabu skill was active."""

    ref: str
    time: str
    project: str
    skill: str
    change: str
    replying_to: str
    text: str


@dataclass
class Decision:
    """One AskUserQuestion question. `pick` is recommended, option, or custom."""

    ref: str
    time: str
    project: str
    skill: str
    change: str
    question: str
    options: list[str]
    answer: str
    pick: str
    notes: str = ""


@dataclass
class Phase:
    """One nabu skill load and the work done until the next load.

    `trigger`: user (slash command), prompted (Skill tool as the first action
    after a user message, such as "build"), agent (Skill tool mid-turn), or
    resumed (the session continues a phase loaded in an earlier copy).
    """

    skill: str
    trigger: str
    tools: dict[str, int] = field(default_factory=dict)
    reviewer_runs: int = 0
    out_tokens: int = 0
    context_tokens: int = 0


@dataclass
class Run:
    """One session that loaded a nabu skill: its skill flow and cost."""

    ref: str
    time: str
    project: str
    changes: list[str]
    phases: list[Phase]
    end: str
    user_messages: int
    questions: int
    # Ref of the session this transcript resumes. Counts as that session, not a new one.
    resumes: str = ""


def read_jsonl(path: pathlib.Path) -> list[dict]:
    """All records of one transcript. Unreadable files count as empty."""
    try:
        return [json.loads(l) for l in path.open() if l.strip()]
    except (OSError, json.JSONDecodeError):
        return []


def text_of(content) -> str:
    """Join text blocks of one message. Handles string and block-list content."""
    if isinstance(content, str):
        return content
    return "\n".join(
        c.get("text", "") for c in content or [] if isinstance(c, dict) and c.get("type") == "text"
    )


def one_line(text: str) -> str:
    return " ".join(text.split())


def parse_ts(value: str) -> datetime.datetime | None:
    try:
        return datetime.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def to_utc(value: str) -> str | None:
    """Normalize an ISO date or time to `YYYY-MM-DDTHH:MM:SSZ`.

    Record timestamps are UTC `Z` strings and are compared as strings, so a
    bound with another offset must be converted first. No offset = UTC.
    """
    at = parse_ts(value)
    if not at:
        return None
    if at.tzinfo is None:
        at = at.replace(tzinfo=datetime.timezone.utc)
    return at.astimezone(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def project_of(project_dir: str, lines: list[dict]) -> str:
    """Project path for a transcript: the cwd whose mangled form names its folder.

    Claude Code names the project folder after the launch cwd with every
    non-alphanumeric character turned into `-`. A record's cwd can be a
    subdirectory or another repo after a `cd`, so only an exact match counts.
    No match -> the folder name itself.
    """
    return next(
        (l["cwd"] for l in lines if l.get("cwd") and re.sub(r"[^A-Za-z0-9]", "-", l["cwd"]) == project_dir),
        project_dir,
    )


def label(project: str) -> str:
    """Short display name: last path segment, or the folder name as is."""
    return pathlib.Path(project).name if project.startswith("/") else project


def transcript_span(path: pathlib.Path) -> tuple[str, str]:
    """(first, last) record timestamp of one transcript, "" when it has none."""
    first = last = ""
    try:
        for raw in path.open():
            ts = (json.loads(raw) if raw.strip() else {}).get("timestamp", "")
            if ts:
                first = first or ts
                last = ts
    except (OSError, json.JSONDecodeError):
        pass
    return first, last


def sessions_in_order() -> list[pathlib.Path]:
    """Main transcripts, oldest session first.

    A resumed session copies the old records with their timestamps, so the
    original and its copy start at the same time. The original ends first.
    Processing in this order credits shared records to the original.
    """
    return sorted(CC_ROOT.glob("*/*.jsonl"), key=lambda p: (*transcript_span(p), str(p)))


_SESSION_PROJECTS: dict[tuple[str, str], str] = {}


def session_project(project_dir: str, session: str) -> str:
    """Project of a main transcript, cached. Subagents inherit it."""
    key = (project_dir, session)
    if key not in _SESSION_PROJECTS:
        _SESSION_PROJECTS[key] = project_of(project_dir, read_jsonl(CC_ROOT / project_dir / f"{session}.jsonl"))
    return _SESSION_PROJECTS[key]


def change_of(text: str, default: str = "?") -> str:
    """Best tablet name in the text: a tablet path beats an `Input context` name.

    Skips template placeholders such as `<change>`.
    """
    found = [
        (i, name)
        for m in CHANGE_RE.finditer(text)
        for i, name in enumerate(m.groups())
        if name and name not in PLACEHOLDER_CHANGES
    ]
    # Free-text briefs glob tablets (`tablets/<name>-*`) => trailing `-` is no part of the name.
    return min(found, key=lambda f: f[0])[1].rstrip("-") if found else default


def final_report(lines: list[dict]) -> str:
    """A subagent's last output: its last text, or a `SubagentHandback` message sent after it."""
    for line in reversed(lines):
        if line.get("type") != "assistant":
            continue
        for b in reversed((line.get("message") or {}).get("content") or []):
            if b.get("type") == "tool_use" and b.get("name") == "SubagentHandback":
                return str((b.get("input") or {}).get("message", ""))
            if b.get("type") == "text" and b.get("text"):
                return b["text"]
    return ""


def parse_report(text: str) -> tuple[str, str, list[Finding]] | None:
    """Parse one final reviewer message. Returns (reviewer, status, findings) or None."""
    head = XML_HEAD_RE.search(text)
    if head:
        findings = []
        for block in XML_FINDING_RE.findall(text):
            tags = {k: one_line(v) for k, v in XML_TAG_RE.findall(block)}
            findings.append(
                Finding(
                    severity=tags.get("severity", ""),
                    problem=tags.get("problem", ""),
                    reference=tags.get("reference", ""),
                    fix=tags.get("fix") or tags.get("alternative", ""),
                )
            )
        status = head.group(2) or ("findings" if findings else "empty")
        return head.group(1), status, findings

    head = MD_HEAD_RE.search(text)
    if not head:
        return None
    status = MD_STATUS_RE.search(text)
    findings = []
    heads = list(MD_FINDING_RE.finditer(text))
    for i, h in enumerate(heads):
        body = text[h.end() : heads[i + 1].start() if i + 1 < len(heads) else len(text)]
        fields = dict(MD_FIELD_RE.findall(body))
        findings.append(
            Finding(
                severity=h.group(1),
                problem=h.group(2).strip(),
                reference=fields.get("loc", ""),
                fix=fields.get("hint", ""),
                evidence=fields.get("evidence", ""),
            )
        )
    return head.group(1) or "?", status.group(1) if status else "?", findings


def turn_reports(lines: list[dict]) -> list[tuple[str, str]]:
    """Each `end_turn` text of one subagent run: its report, plus one per `SendMessage`
    follow-up. The stop reason marks a finished report => a preamble before tool calls, or
    a turn still running at mining time, never passes as one."""
    out = []
    for line in lines:
        message = line.get("message") or {}
        if line.get("type") != "assistant" or message.get("stop_reason") != "end_turn":
            continue
        content = message.get("content")
        blocks = content if isinstance(content, list) else []
        texts = [b["text"] for b in blocks if b.get("type") == "text" and b.get("text", "").strip()]
        # Own time per turn: a follow-up after `mark` must not re-emit the first report.
        out += [(line.get("timestamp", ""), t) for t in texts[-1:]]
    return out


def skill_at(transcript: pathlib.Path, at: str) -> str | None:
    """Nabu skill active in the session at time `at`: the last one loaded before it.

    By time, not by spawning tool call => nested runs (a subagent calling the duck) and
    background runs resolve the same way. A skill loaded inside a subagent is missed.
    """
    skill = None
    for line in read_jsonl(transcript):
        if line.get("timestamp", "") > at:
            break
        content = (line.get("message") or {}).get("content")
        if line.get("type") == "user" and (command := COMMAND_RE.search(text_of(content))):
            skill = command.group(1)
        for b in content if isinstance(content, list) else []:
            if b.get("type") == "tool_use" and b.get("name") == "Skill" and (
                s := SKILL_RE.search(str((b.get("input") or {}).get("skill", "")))
            ):
                skill = s.group(1)
    return skill


def duck_reports(path: pathlib.Path, lines: list[dict]) -> list[Report]:
    """One report per duck turn. The duck reviews like a verify lane, in any session =>
    every run counts, labeled with the nabu skill active when it started (`none`).

    The brief is no skill source: it often names skill paths (`skills/nabu-learn`) as
    its subject. No change named -> the session stands in => distinct-change counts work.
    """
    stamps = [l["timestamp"] for l in lines if l.get("timestamp")]
    session_dir = path.parent.parent
    skill = skill_at(session_dir.with_suffix(".jsonl"), stamps[0]) if stamps else None
    if skill in IGNORED_PHASES:
        return []
    prompt = next((text_of(l["message"].get("content")) for l in lines if l.get("type") == "user"), "")
    return [
        Report(
            ref=f"cc:{session_dir.parent.name}/{session_dir.name}/{path.stem}",
            time=time,
            project=session_project(session_dir.parent.name, session_dir.name),
            skill=skill or "none",
            reviewer=DUCK_AGENT,
            status="report",
            change=change_of(prompt, f"session-{session_dir.name[:8]}"),
            findings=[Finding(severity="report", problem=text)],
        )
        for time, text in turn_reports(lines)
    ]


def reports() -> list[Report]:
    """Reviewer reports from subagent transcripts.

    A resumed session copies its subagent transcripts, so one agent id can
    appear under several sessions. The copy under the oldest session wins.
    """
    rank = {(p.parent.name, p.stem): i for i, p in enumerate(sessions_in_order())}
    paths = sorted(
        CC_ROOT.glob("*/*/subagents/agent-*.jsonl"),
        key=lambda p: (rank.get((p.parent.parent.parent.name, p.parent.parent.name), len(rank)), str(p)),
    )
    out, seen = [], set()
    for path in paths:
        if path.stem in seen:
            continue
        seen.add(path.stem)
        lines = read_jsonl(path)
        try:
            meta = json.loads(path.with_name(f"{path.stem}.meta.json").read_text())
        except (OSError, json.JSONDecodeError):
            meta = {}  # older agents have no meta
        if meta.get("agentType") == DUCK_AGENT:
            out += duck_reports(path, lines)
            continue
        prompt = next(
            (text_of(l["message"].get("content")) for l in lines if l.get("type") == "user"), ""
        )
        parsed = parse_report(final_report(lines))
        if not parsed:
            continue
        skill = SKILL_RE.search(prompt)
        # A prompt naming no nabu skill is another workflow's reviewer (runs of the old workflow).
        if not skill or skill.group(1) in IGNORED_PHASES:
            continue
        session_dir = path.parent.parent
        out.append(
            Report(
                ref=f"cc:{session_dir.parent.name}/{session_dir.name}/{path.stem}",
                # Last record: a reviewer still running at mining time lands in the next window.
                time=next((l["timestamp"] for l in reversed(lines) if l.get("timestamp")), ""),
                project=session_project(session_dir.parent.name, session_dir.name),
                skill=skill.group(1),
                reviewer=parsed[0],
                status=parsed[1],
                change=change_of(prompt),
                findings=parsed[2],
            )
        )
    return out


def human_text(text: str) -> str:
    """The user's own words in one message, or "" for harness-generated messages."""
    text = NOISE_RE.sub("", text)
    text = SELECTION_RE.sub(
        lambda m: f"[selection] {one_line(m.group(1))[:SELECTION_CHARS]}\n", text
    ).strip()
    if not text or text.startswith(NOT_HUMAN_PREFIXES):
        return ""
    return text[:MESSAGE_CHARS]


def decision_of(result: dict, base: dict) -> list[Decision]:
    """Questions plus answers from one AskUserQuestion result."""
    answers = result.get("answers") or {}
    notes = result.get("annotations") or {}
    out = []
    for q in result.get("questions") or []:
        text = q.get("question", "")
        labels = [o.get("label", "") for o in q.get("options") or []]
        answer = str(answers.get(text, ""))
        if not answer:
            continue
        picked = [l for l in labels if l == answer or l in answer.split(", ")]
        if any("(Recommended)" in l for l in picked):
            pick = "recommended"
        else:
            pick = "option" if picked else "custom"
        out.append(
            Decision(
                **base,
                question=text,
                options=labels,
                answer=answer,
                pick=pick,
                notes=(notes.get(text) or {}).get("notes", "") if isinstance(notes, dict) else "",
            )
        )
    return out


def backfill_changes(items: list) -> None:
    """Give `?` items the next change named later in the session.

    Outline and inscribe ask questions before any change path exists.
    """
    following = "?"
    for item in reversed(items):
        if item.change == "?":
            item.change = following
        else:
            following = item.change


def session_signals(
    path: pathlib.Path, seen_records: dict[str, str]
) -> tuple[list[Feedback], list[Decision], Run | None]:
    """Walk one main transcript. Collect feedback, answers, and the skill flow.

    Tracks the active nabu skill (last one loaded) and the change under work.
    Nothing is collected before the first nabu skill loads. Records already
    in `seen_records` (uuid -> owning session ref) come from a resumed copy of
    another session: they emit nothing, and the run records which session it resumes.
    """
    feedback: list[Feedback] = []
    decisions: list[Decision] = []
    ordered: list = []
    phases: list[Phase] = []
    resumes = ""
    last_seen_at: dict[str, datetime.datetime] = {}
    # Streamed replies repeat one message id across records, each with the same usage.
    seen_message_ids: set[str] = set()
    skill, change, last_text = None, "?", ""
    # True from a user message until the first tool result of the agent's turn.
    turn_opened_by_user = False
    lines = read_jsonl(path)
    project = project_of(path.parent.name, lines)
    ref = f"cc:{path.parent.name}/{path.stem}"

    def base(line: dict) -> dict:
        return dict(
            ref=ref,
            time=line.get("timestamp", ""),
            project=project,
            skill=skill or "?",
            change=change,
        )

    def add_feedback(line: dict, raw: str) -> None:
        nonlocal turn_opened_by_user
        text = human_text(raw)
        turn_opened_by_user = turn_opened_by_user or bool(text)
        at = parse_ts(line.get("timestamp", ""))
        prev = last_seen_at.get(text)
        echo = bool(prev and at and (at - prev).total_seconds() < DUPLICATE_WINDOW_S)
        if at:
            last_seen_at[text] = at
        if skill and text and not echo and not copied:
            item = Feedback(
                **base(line),
                replying_to=one_line(last_text)[-REPLY_CONTEXT_CHARS:],
                text=text,
            )
            feedback.append(item)
            ordered.append(item)

    def enter(new_skill: str, trigger: str) -> None:
        nonlocal skill
        skill = None if new_skill in IGNORED_PHASES else new_skill
        if skill and not copied:
            phases.append(Phase(skill, trigger))

    def phase() -> Phase | None:
        """Phase that owns the current record. None before any nabu skill load or for copies."""
        if not skill or copied:
            return None
        if not phases:
            phases.append(Phase(skill, "resumed"))
        return phases[-1]

    for line in lines:
        # Copied records still update skill and change state, but emit nothing.
        uid = line.get("uuid")
        copied = bool(uid) and uid in seen_records
        if copied:
            resumes = resumes or seen_records[uid]
        elif uid:
            seen_records[uid] = ref
        kind = line.get("type")
        content = (line.get("message") or {}).get("content")
        blocks = content if isinstance(content, list) else []

        if kind == "assistant":
            message = line.get("message") or {}
            if (p := phase()) and message.get("id") not in seen_message_ids:
                seen_message_ids.add(message.get("id"))
                usage = message.get("usage") or {}
                p.out_tokens += usage.get("output_tokens", 0)
                p.context_tokens += sum(
                    usage.get(k, 0)
                    for k in (
                        "input_tokens",
                        "cache_read_input_tokens",
                        "cache_creation_input_tokens",
                    )
                )
            for b in blocks:
                if b.get("type") == "text" and b.get("text"):
                    last_text = b["text"]
                elif b.get("type") == "tool_use":
                    inp = b.get("input") or {}
                    if b.get("name") == "Skill" and (s := SKILL_RE.search(inp.get("skill", ""))):
                        enter(s.group(1), "prompted" if turn_opened_by_user else "agent")
                        arg = KEBAB_ARG_RE.match(str(inp.get("args", "")))
                        change = arg.group(1) if arg else change
                    elif p := phase():
                        name = b.get("name", "?")
                        p.tools[name] = p.tools.get(name, 0) + 1
                        # Reviewer prompts name a nabu instruction file.
                        p.reviewer_runs += name == "Agent" and (
                            inp.get("subagent_type") == DUCK_AGENT or bool(SKILL_RE.search(json.dumps(inp)))
                        )
                    change = change_of(json.dumps(inp), change)
            continue

        # Mid-turn messages arrive as queued attachments, not user records.
        attachment = line.get("attachment") or {}
        if (
            kind == "attachment"
            and attachment.get("type") == "queued_command"
            and (attachment.get("origin") or {}).get("kind") == "human"
        ):
            add_feedback(line, text_of(attachment.get("prompt")))
            continue

        if kind != "user" or line.get("isMeta"):
            continue
        raw = text_of(content)
        if command := COMMAND_RE.search(raw):
            enter(command.group(1), "user")
            args = COMMAND_ARGS_RE.search(raw)
            arg = KEBAB_ARG_RE.match(args.group(1)) if args else None
            change = arg.group(1) if arg else change
            continue
        result = line.get("toolUseResult")
        if isinstance(result, dict) and "questions" in result:
            answered = decision_of(result, base(line)) if skill and not copied else []
            decisions.extend(answered)
            ordered.extend(answered)
            continue
        if any(b.get("type") == "tool_result" for b in blocks):
            turn_opened_by_user = False
        else:
            add_feedback(line, raw)
    backfill_changes(ordered)
    run = None
    if phases:
        changes = list(dict.fromkeys(i.change for i in ordered if i.change != "?"))
        if change != "?" and change not in changes:
            changes.append(change)
        run = Run(
            ref=ref,
            time=next((l["timestamp"] for l in lines if l.get("timestamp")), ""),
            project=project,
            changes=changes,
            phases=phases,
            end=next((l["timestamp"] for l in reversed(lines) if l.get("timestamp")), ""),
            user_messages=len(feedback),
            questions=len(decisions),
            resumes=resumes,
        )
    return feedback, decisions, run


def all_signals() -> tuple[list[Feedback], list[Decision], list[Run]]:
    feedback, decisions, runs, seen_records = [], [], [], {}
    for path in sessions_in_order():
        f, d, r = session_signals(path, seen_records)
        feedback += f
        decisions += d
        runs += [r] if r else []
    return feedback, decisions, runs


def since_of(args) -> str | None:
    """Lower time bound: --since, else `last_run` from the state file. --all = no bound."""
    if args.all:
        return None
    if args.since:
        since = to_utc(args.since)
        if not since:
            sys.exit(f"--since must be an ISO date or time, got {args.since!r}")
        return since
    try:
        return to_utc(json.loads(STATE_PATH.read_text()).get("last_run") or "")
    except (OSError, json.JSONDecodeError):
        return None


def keep(item, args, since: str | None) -> bool:
    skills = [p.skill for p in item.phases] if isinstance(item, Run) else [item.skill]
    # A session that started before `since` but kept going still counts.
    latest = item.end if isinstance(item, Run) else item.time
    # ISO strings compare chronologically, and a bare date sorts before its own times.
    return (
        (not since or latest >= since)
        and (not args.project or args.project in item.project or args.project in item.ref)
        and (not args.skill or args.skill in skills)
    )


def emit(items: list, args, header, block) -> int:
    """Shared output: JSON lines, or a count header plus one Markdown block per item."""
    since = since_of(args)
    items = sorted((i for i in items if keep(i, args, since)), key=lambda i: (i.time, i.ref))
    if args.json:
        for i in items:
            print(json.dumps(asdict(i)))
        return 0
    print(f"since: {since or 'all time'} · mined_at: {utc_now()}")
    header(items)
    for i in items:
        block(i)
    return 0


def title(i, *tail: str) -> str:
    return " · ".join([f"\n### {i.time[:10]}", label(i.project), f"`{i.change}`", *tail])


def cmd_findings(args) -> int:
    def header(rs):
        stats = collections.Counter((r.skill, r.reviewer, r.status) for r in rs)
        print(f"# {len(rs)} reviewer runs, {sum(len(r.findings) for r in rs)} findings\n")
        for (skill, reviewer, status), n in sorted(stats.items()):
            print(f"- {skill}/{reviewer} {status}: {n}")

    def block(r):
        for f in r.findings:
            print(title(r, f"{r.skill}/{r.reviewer}", f.severity))
            print(f"ref: {r.ref}")
            # Indented: a multi-line duck report must not open headings of its own.
            print(f"problem: {f.problem}".replace("\n", "\n  "))
            for label, value in (("at", f.reference), ("evidence", f.evidence), ("fix", f.fix)):
                if value:
                    print(f"{label}: {value}")

    return emit(reports(), args, header, block)


def cmd_feedback(args) -> int:
    def header(fs):
        stats = collections.Counter(f.skill for f in fs)
        print(f"# {len(fs)} user messages\n")
        for skill, n in sorted(stats.items()):
            print(f"- during {skill}: {n}")

    def block(f):
        print(title(f, f"during {f.skill}"))
        print(f"ref: {f.ref}")
        if f.replying_to:
            print(f"agent: …{f.replying_to}")
        print(f"user: {f.text}")

    return emit(all_signals()[0], args, header, block)


def cmd_decisions(args) -> int:
    def header(ds):
        stats = collections.Counter((d.skill, d.pick) for d in ds)
        print(f"# {len(ds)} answered questions\n")
        for (skill, pick), n in sorted(stats.items()):
            print(f"- {skill} {pick}: {n}")

    def block(d):
        print(title(d, d.skill, f"pick: {d.pick}"))
        print(f"ref: {d.ref}")
        print(f"q: {one_line(d.question)}")
        print(f"options: {' | '.join(d.options)}")
        print(f"answer: {one_line(d.answer)}")
        if d.notes:
            print(f"notes: {one_line(d.notes)}")

    return emit(all_signals()[1], args, header, block)


def kilo(n: int) -> str:
    return f"{n / 1000:.0f}k"


def cmd_runs(args) -> int:
    runs = all_signals()[2]

    def header(rs):
        # Who started each skill after the first. User-started steps hint at a broken handoff.
        steps = collections.Counter(
            (a.skill, b.skill, b.trigger)
            for r in rs
            for a, b in zip(r.phases, r.phases[1:])
        )
        print(f"# {len(rs)} sessions\n")
        for (a, b, trigger), n in steps.most_common():
            print(f"- {a} -> {b} by {trigger}: {n}")

    def block(r):
        print(" · ".join([f"\n### {r.time[:10]}", label(r.project), ", ".join(r.changes) or "?"]))
        print(f"ref: {r.ref}")
        if r.resumes:
            print(f"resumes: {r.resumes} (same session, do not count twice)")
        print("flow: " + " -> ".join(f"{p.skill}({p.trigger})" for p in r.phases))
        for p in r.phases:
            top = ", ".join(f"{name} {n}" for name, n in collections.Counter(p.tools).most_common(4))
            print(
                f"- {p.skill}: {sum(p.tools.values())} tools ({top}),"
                f" {kilo(p.out_tokens)} out, {kilo(p.context_tokens)} cumulative input"
            )
        print(
            f"signals: {sum(p.reviewer_runs for p in r.phases)} reviewer spawns,"
            f" {r.user_messages} user messages,"
            f" {r.questions} questions"
        )

    return emit(runs, args, header, block)


def edit_line(path: str | None, new_text: str | None) -> str:
    """One parent edit: path plus the start of the new text, enough to match it to a finding."""
    return f"[edit] {path or '?'}: {one_line(new_text or '')[:EDIT_SNIPPET_CHARS]}"


def parent_lines(ref: str) -> list[str]:
    """Parent activity after the reviewer's final report reached the parent session.

    Delivery shapes, all keyed by the agent id (older agents have no
    `.meta.json`, so the id is the only stable key):
    - background (most): an immediate `async_launched` result, then a task
      notification per stop. The window starts at the last one.
    - hand-back: an `<agent-message from="<agent id>">` user message.
    - foreground: a `completed` tool result.
    - nested (a subagent called it): a tool result in the caller's file that ends with
      `agentId: <agent id> (use SendMessage ...`. No `toolUseResult` there.
    """
    project, session, agent = ref.split("/")
    agent_id = agent.removeprefix("agent-")
    markers = (f"<task-id>{agent_id}</task-id>", f'from=\\"{agent_id}\\"', f"agentId: {agent_id} (")
    transcript = CC_ROOT / project / f"{session}.jsonl"
    if not transcript.exists():
        raise ValueError(f"no parent transcript at {transcript}")

    def delivers_report(line: dict) -> bool:
        raw = json.dumps(line.get("message") or line.get("attachment") or {})
        result = line.get("toolUseResult")
        return any(m in raw for m in markers) or (
            isinstance(result, dict)
            and result.get("status") == "completed"
            and result.get("agentId") == agent_id
        )

    # A nested run (a subagent calling the duck) reports to its caller's file, not the session's.
    subagents = transcript.with_suffix("") / "subagents"
    try:
        caller = json.loads((subagents / f"{agent}.meta.json").read_text()).get("parentAgentId")
    except (OSError, json.JSONDecodeError):
        caller = None  # older agents have no meta
    lines = read_jsonl(subagents / f"agent-{caller}.jsonl" if caller else transcript)
    start = max((i for i, l in enumerate(lines) if delivers_report(l)), default=None)
    if start is None:
        raise ValueError(f"the report never reached {'subagent ' + caller if caller else 'the parent session'}")

    out = []
    for line in lines[start + 1 :]:
        if line.get("isMeta"):
            continue  # skill bodies, re-invocation notes: harness text, not the user
        role = line.get("type")
        content = (line.get("message") or {}).get("content")
        attachment = line.get("attachment") or {}
        if role == "attachment" and (attachment.get("origin") or {}).get("kind") == "human":
            out.append(f"[user] {text_of(attachment.get('prompt'))}")
            continue
        if role == "user" and isinstance(content, str):
            # Other notifications echo whole reports. Keep only the summary.
            note = NOTIFICATION_RE.search(content)
            if note:
                out.append(f"[notification] {note.group(1)}")
            elif text := human_text(content):
                out.append(f"[user] {text}")
            continue
        for b in content if isinstance(content, list) else []:
            if b.get("type") == "text" and b.get("text"):
                text = b["text"] if role == "assistant" else human_text(b["text"])
                if text:
                    out.append(f"[{role}] {text}")
            elif b.get("type") == "tool_use" and b.get("name") in ("Edit", "Write"):
                inp = b.get("input") or {}
                out.append(edit_line(inp.get("file_path"), inp.get("new_string") or inp.get("content")))
    return out


def cmd_parent(args) -> int:
    try:
        lines = parent_lines(args.ref.removeprefix("cc:"))
    except (OSError, ValueError, TypeError) as exc:
        print(f"cannot read parent of {args.ref}: {exc}", file=sys.stderr)
        return 1
    budget = PARENT_CHAR_BUDGET
    for line in lines:
        if budget <= 0:
            print("[truncated]")
            break
        print(html.unescape(line[:budget]))
        budget -= len(line)
    return 0


RECEIPTS_PATH = STATE_PATH.parent / "receipts.md"
RULES_PATH = SKILLS_ROOT / "nabu-inscribe" / "learned-rules.md"
RULES_BUDGET = 30
# `watch` = seen in one session only. A second session promotes it to `proposed`.
CANDIDATE_STATUSES = {"watch", "proposed", "applied", "declined"}
ACCEPTED = {"adopted", "fixed", "deferred", "user"}
REJECTED = {"rejected"}
PROCESS = {"observed", "prompted"}
# Which `###` IDs each receipts section takes. Retired keeps tombstones so IDs never repeat.
SECTION_KINDS = {"Rules": "L", "Watchlist": None, "Candidates": "C", "Retired": "L"}
# Each Design rule belongs to the plan-time reviewer that checks it.
OWNER_HEADINGS = {"Opponent", "Artifacts"}
TARGET_RE = re.compile(r"^- Target: `([^`]+)` :: (.+)$", re.M)
# A rule line in any nabu skill file ends with its ID: "... (L7)".
RULE_LINE_RE = re.compile(r"^\s*- .*\(L(\d+)\)\s*$", re.M)
ENTRY_RE = re.compile(r"^### (?:([LC])(\d+): )?(.+)$", re.M)
DATE_FIELD_RE = re.compile(r"^- (Introduced|Last edited|Status): (.*)$", re.M)
EVIDENCE_RE = re.compile(
    r"^\s+- (\d{4}-\d{2}-\d{2}) · [^·]+ · `([^`]+)` · [^·]+ · `(cc:[^`]+)` · (\S+)", re.M
)


def sections(text: str) -> dict[str, str]:
    """`## Heading` -> body, for the receipts file."""
    parts = re.split(r"^## (.+)$", text, flags=re.M)
    return {parts[i].strip(): parts[i + 1] for i in range(1, len(parts) - 1, 2)}


def entries(body: str) -> list[tuple[str | None, int | None, str, str]]:
    """(kind L|C|None, id, title, entry body) for each `###` entry of one section."""
    heads = list(ENTRY_RE.finditer(body))
    out = []
    for i, h in enumerate(heads):
        end = heads[i + 1].start() if i + 1 < len(heads) else len(body)
        out.append((h.group(1), int(h.group(2)) if h.group(2) else None, h.group(3), body[h.end() : end]))
    return out


def rule_owners(text: str) -> tuple[dict[int, str], list[str]]:
    """Rule id -> owning heading in learned-rules.md, plus unowned-rule errors.

    `## Design` rules sit under an owner `###` heading. `## Defaults` rules
    have no reviewer, so their owner is `Defaults`.
    """
    owners, errors, h2, h3 = {}, [], "", ""
    for line in text.splitlines():
        if line.startswith("## "):
            h2, h3 = line[3:].strip(), ""
        elif line.startswith("### "):
            h3 = line[4:].strip()
        elif m := RULE_LINE_RE.match(line):
            num = int(m.group(1))
            if h2 == "Design" and h3 not in OWNER_HEADINGS:
                errors.append(f"L{num}: Design rule outside an owner heading ({', '.join(sorted(OWNER_HEADINGS))})")
            owners[num] = h3 if h2 == "Design" else h2
    return owners, errors


def is_date(value: str) -> bool:
    try:
        datetime.date.fromisoformat(value)
        return True
    except ValueError:
        return False


def ref_exists(ref: str) -> bool:
    """`cc:<project>/<session>[/<agent>]` points at a transcript still on disk."""
    parts = ref.removeprefix("cc:").split("/")
    if len(parts) == 2:
        return (CC_ROOT / parts[0] / f"{parts[1]}.jsonl").exists()
    return len(parts) == 3 and (CC_ROOT / parts[0] / parts[1] / "subagents" / f"{parts[2]}.jsonl").exists()


def lint_receipts(receipts: str | None, rule_files: dict[pathlib.Path, str]) -> tuple[list[str], list[str]]:
    """Pure core of `lint`: (errors, warnings) for receipts against the rule files.

    `None` = no receipts file: only the rule-file checks run.
    """
    errors, warnings = [], []
    if receipts is None:
        warnings.append("no receipts.md: receipt checks skipped")
    live = {}  # rule id -> files whose lines carry it
    for path, text in rule_files.items():
        for m in RULE_LINE_RE.finditer(text):
            live.setdefault(int(m.group(1)), []).append(path)
    owners, owner_errors = rule_owners(rule_files.get(RULES_PATH, ""))
    errors += owner_errors
    by_section = sections(receipts or "")
    seen_ids: dict[tuple[str, int], str] = {}

    for section, body in by_section.items():
        for kind, num, title, entry in entries(body):
            name = f"{kind}{num}" if kind else title
            if kind and (kind, num) in seen_ids:
                errors.append(f"{name}: ID reused (also in {seen_ids[(kind, num)]})")
            if kind:
                seen_ids[(kind, num)] = section
            if section not in SECTION_KINDS:
                errors.append(f"{name}: unknown section `{section}`")
                continue
            if kind != SECTION_KINDS[section]:
                want = SECTION_KINDS[section] or "no ID"
                errors.append(f"{name}: entries under `{section}` take {want}")
            if section == "Retired":
                continue  # tombstone: only its ID matters
            if len(TARGET_RE.findall(entry)) != 1:
                errors.append(f"{name}: needs exactly one `- Target: `<path>` :: <heading>` line")
            fields = dict(DATE_FIELD_RE.findall(entry))
            evidence = EVIDENCE_RE.findall(entry)
            for date, _change, ref, disposition in evidence:
                if not is_date(date):
                    errors.append(f"{name}: bad evidence date {date}")
                if disposition not in ACCEPTED | REJECTED | PROCESS:
                    errors.append(f"{name}: unknown disposition `{disposition}`")
                if not ref_exists(ref):
                    # Claude Code prunes old transcripts. The quote stays valid without them.
                    warnings.append(f"{name}: transcript gone for {ref}")
            if section == "Candidates":
                status, _, date = fields.get("Status", "").partition(" · ")
                if status not in CANDIDATE_STATUSES or not is_date(date):
                    errors.append(f"{name}: Status must be `<{'|'.join(sorted(CANDIDATE_STATUSES))}> · <date>`")
                continue
            introduced, edited = fields.get("Introduced", ""), fields.get("Last edited", "")
            if not (is_date(introduced) and is_date(edited)):
                errors.append(f"{name}: needs Introduced and Last edited dates")
            elif edited < introduced:
                errors.append(f"{name}: Last edited is before Introduced")
            if not evidence:
                errors.append(f"{name}: no evidence lines in the receipt format")
            if section == "Rules":
                # A rule needs 2+ changes of one polarity: accepted, or rejected for a
                # `Do not challenge` line. Process observations back candidates only.
                def changes_with(kinds: set[str]) -> set[str]:
                    return {c for _, c, _, d in evidence if d in kinds}
                best = max(len(changes_with(ACCEPTED)), len(changes_with(REJECTED)))
                if best < 2:
                    errors.append(f"{name}: {best} distinct change(s) of one disposition, rules need 2+")
                if num not in live:
                    errors.append(f"{name}: no rule line ends with (L{num}) in any nabu skill file")
                target = TARGET_RE.search(entry)
                if target and num in live and SKILLS_ROOT / target.group(1) not in live[num]:
                    errors.append(f"{name}: Target is `{target.group(1)}`, but the rule line is elsewhere")
                if target and target.group(1).endswith("learned-rules.md") and num in owners:
                    if target.group(2).strip() != owners[num]:
                        errors.append(f"{name}: Target says `{target.group(2).strip()}`, rule sits under `{owners[num]}`")

    receipted = {num for (kind, num), section in seen_ids.items() if kind == "L" and section == "Rules"}
    retired = {num for (kind, num), section in seen_ids.items() if kind == "L" and section == "Retired"}
    for num in sorted(retired & set(live)):
        errors.append(f"L{num}: retired, but a rule line still carries it")
    for num, paths in sorted(live.items()):
        if receipts is not None and num not in receipted:
            errors.append(f"L{num}: rule line has no receipt ({', '.join(p.name for p in paths)})")
        if len(paths) > 1:
            errors.append(f"L{num}: rule line appears {len(paths)} times. One source per rule")
    rules_text = rule_files.get(RULES_PATH, "")
    count = len(RULE_LINE_RE.findall(rules_text))
    if count > RULES_BUDGET:
        errors.append(f"learned-rules.md holds {count} rules, budget {RULES_BUDGET}: merge classes")
    return errors, warnings


def cmd_lint(args) -> int:
    # `nabu-verify/implementer-rules.md` is a symlink to the with-implementer file => resolve
    # to count each rule line once.
    rule_files = {
        p.resolve(): p.read_text()
        for p in sorted([*SKILLS_ROOT.glob("nabu-*/**/*.md"), *SKILLS_ROOT.glob("with-implementer/**/*.md")])
        if p.parent != STATE_PATH.parent
    }
    # receipts.md is gitignored (it quotes private sessions) => a fresh clone has none.
    receipts = RECEIPTS_PATH.read_text() if RECEIPTS_PATH.exists() else None
    errors, warnings = lint_receipts(receipts, rule_files)
    for w in warnings:
        print(f"WARN  {w}")
    for e in errors:
        print(f"ERROR {e}")
    print(f"{len(errors)} error(s), {len(warnings)} warning(s)")
    return 1 if errors else 0


def utc_now() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def cmd_mark(args) -> int:
    """Record this run, so the next run mines only newer sessions.

    `--at` takes the `mined_at` time the mining commands printed. Sessions that
    logged while the run was in progress then stay inside the next window.
    """
    at = to_utc(args.at) if args.at else utc_now()
    if not at:
        print(f"--at must be an ISO time, got {args.at!r}", file=sys.stderr)
        return 1
    STATE_PATH.write_text(json.dumps({"last_run": at}, indent=2) + "\n")
    print(f"last_run = {at}")
    return 0


def main() -> int:
    # Agents pipe the output into `head`. Exit quietly when the pipe closes.
    signal.signal(signal.SIGPIPE, signal.SIG_DFL)
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = parser.add_subparsers(dest="cmd", required=True)
    for name, run, help_text in (
        ("findings", cmd_findings, "reviewer findings"),
        ("feedback", cmd_feedback, "user messages while a nabu skill was active"),
        ("decisions", cmd_decisions, "AskUserQuestion answers"),
        ("runs", cmd_runs, "skill flow and cost per session"),
    ):
        p = sub.add_parser(name, help=help_text)
        p.add_argument("--since", help="ISO date or time, inclusive. Default: last-run.json")
        p.add_argument("--all", action="store_true", help="ignore last-run.json, mine everything")
        p.add_argument("--project", help="substring of the project path or the Claude Code folder name in the ref")
        p.add_argument("--skill", help="skill phase: inscribe, outline, verify, build, tweak, ...")
        p.add_argument("--json", action="store_true", help="one JSON object per line")
        p.set_defaults(run=run)
    p = sub.add_parser("parent", help="parent activity after one reviewer report")
    p.add_argument("ref", help="ref from `findings`: cc:<project>/<session>/<agent>")
    p.set_defaults(run=cmd_parent)
    m = sub.add_parser("mark", help="overwrite last-run.json")
    m.add_argument("--at", help="ISO time, normally the mined_at of this run. Default: now")
    m.set_defaults(run=cmd_mark)
    sub.add_parser("lint", help="check rule files against receipts.md").set_defaults(run=cmd_lint)
    args = parser.parse_args()
    return args.run(args)


if __name__ == "__main__":
    sys.exit(main())
