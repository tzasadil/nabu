#!/usr/bin/env python3
"""Mechanical bundle check for one nabu folder (a plain tablet or one slice), run before the plan reviewers.

Why: the artifacts reviewer kept spending findings on defects a script can see
(L15: task paths and commands that do not resolve, L16: `UNCERTAIN` premises
outside Gotchas or tied to no task). Catching them first leaves the reviewer
for judgment calls.

Usage: check_bundle.py <folder>
Exit 1 when any error is printed, 2 when the folder has no tasks.md (a stacked tablet root or a typo). Warnings never fail the run.
"""

from __future__ import annotations

import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

BACKTICK_RE = re.compile(r"`([^`\n]+)`")
TASK_RE = re.compile(r"^\s*- \[[ x]\] (\d+\.\d+)\b", re.M)
TASK_REF_RE = re.compile(r"\b(?:[Tt]asks? )?(\d+\.\d+)\b")
SECTION_RE = re.compile(r"^## ", re.M)
# jj change IDs use the k-z alphabet. 8+ chars keeps short words out.
CHANGE_ID_RE = re.compile(r"^[k-z]{8,32}$")
FILE_EXT_RE = re.compile(r"\.(ts|tsx|js|jsx|mjs|cjs|json|md|ya?ml|sql|sh|py|prisma|css|scss|html|toml|csv|xlsx|cs|csproj|go|rs|vue|svelte|example|lock)$")
# A task that creates a path may name one that does not exist yet.
CREATE_RE = re.compile(r"\b(new|add|adds|create|creates|write|writes|generate|move|moves|rename|renames)\b", re.I)
# Preamble-wide working directory ("Run backend commands from `apps/x`").
WORKDIR_RE = re.compile(r"(working directory|\bfrom (the )?(repo|repository|workspace)( root)?\b|\bfrom `|\bcd |-C )", re.I)
# Per-line working directory ("`pnpm test` in `apps/web`", "in the same directory").
LINE_WORKDIR_RE = re.compile(r"(\b(in|from) `[^`]+`|same directory|repository root|repo root|\bcd |-C |--dir |--prefix |--cwd |-R )", re.I)
COMMAND_RE = re.compile(r"^(bash|sh|pnpm|npm|npx|yarn|just|make|uv|python3?|node|go|cargo|dotnet|nabu|rg|fd|jj|docker|vitest|pytest)(\s|$)")
IMPORT_EXTS = (".ts", ".tsx", ".js", ".mjs")
SKIP_DIRS = {".git", ".jj", "node_modules", "dist", "build", "target", ".next", ".turbo", "coverage", "__pycache__", ".venv"}


@dataclass(frozen=True)
class Issue:
    level: str  # "error" | "warning"
    file: str
    line: int
    message: str


def looks_like_path(token: str) -> bool:
    if " " in token or any(c in token for c in "$<>{}*?|=()[]"):
        return False
    if token.startswith(("http://", "https://", "@", "-", "origin/", "upstream/")) or "::" in token or "…" in token or "..." in token:
        return False
    # A leading slash is a route (`/responses`) unless its first segment exists on disk.
    if token.startswith("/") and not Path("/" + token.strip("/").split("/", 1)[0]).exists():
        return False
    return "/" in token or bool(FILE_EXT_RE.search(token))


def resolves(token: str, bases: list[Path], suffixes: set[str]) -> bool:
    clean = token.rstrip("/").split(":", 1)[0].removeprefix("./")
    # Import specifiers omit the extension (`apps/x/src/x.module`).
    candidates = [clean, *(clean + ext for ext in IMPORT_EXTS)]
    if any((base / c).exists() for base in bases for c in candidates):
        return True
    # Partial paths ("knowledge-source/domain/") resolve when some repo path ends with them.
    return any(c in suffixes for c in candidates)


def check_paths(tasks: str, bases: list[Path], suffixes: set[str]) -> list[Issue]:
    issues = []
    for n, line in enumerate(tasks.splitlines(), 1):
        for token in BACKTICK_RE.findall(line):
            if not looks_like_path(token) or COMMAND_RE.match(token) or token.startswith("/tmp/"):
                continue
            # Extensionless and rooted nowhere => a route (`admin/directory/owners`), not a path.
            head = token.strip("/").split("/", 1)[0]
            if not FILE_EXT_RE.search(token) and not any((base / head).exists() for base in bases):
                continue
            if resolves(token, bases, suffixes):
                continue
            parent = token.rstrip("/").rsplit("/", 1)[0] if "/" in token.rstrip("/") else ""
            if CREATE_RE.search(line) and (not parent or resolves(parent, bases, suffixes)):
                continue
            issues.append(Issue("error", "tasks.md", n, f"path does not resolve: `{token}`"))
    return issues


def check_workdir(tasks: str) -> list[Issue]:
    if WORKDIR_RE.search(tasks.split("\n## ", 1)[0]):
        return []
    issues = []
    for n, line in enumerate(tasks.splitlines(), 1):
        commands = [t for t in BACKTICK_RE.findall(line) if COMMAND_RE.match(t)]
        if commands and not LINE_WORKDIR_RE.search(line):
            issues.append(Issue("error", "tasks.md", n, f"command with no working directory: `{commands[0]}`"))
    return issues


def check_change_ids(tasks: str, blueprint: str, resolve_id) -> list[Issue]:
    issues = []
    for name, text in (("tasks.md", tasks), ("blueprint.md", blueprint)):
        for n, line in enumerate(text.splitlines(), 1):
            for token in BACKTICK_RE.findall(line):
                if CHANGE_ID_RE.match(token) and not resolve_id(token):
                    issues.append(Issue("error", name, n, f"jj change ID does not resolve: `{token}`"))
    return issues


def sections(text: str) -> list[tuple[str, int, str]]:
    """(heading, first line number, body) per `## ` section. Preamble has heading ''."""
    out, heading, start, body = [], "", 1, []
    for n, line in enumerate(text.splitlines(), 1):
        if line.startswith("## "):
            out.append((heading, start, "\n".join(body)))
            heading, start, body = line[3:].strip(), n, []
        else:
            body.append(line)
    out.append((heading, start, "\n".join(body)))
    return out


def check_uncertain(blueprint: str, task_ids: set[str]) -> list[Issue]:
    issues = []
    for heading, start, body in sections(blueprint):
        in_gotchas = heading.lower().startswith("gotchas")
        for offset, line in enumerate(body.splitlines(), 1):
            if "UNCERTAIN" not in line:
                continue
            n = start + offset
            if not in_gotchas:
                issues.append(Issue("error", "blueprint.md", n, f"`UNCERTAIN` outside Gotchas (section: {heading or 'preamble'})"))
                continue
            refs = set(TASK_REF_RE.findall(line))
            if not refs:
                issues.append(Issue("warning", "blueprint.md", n, "Gotchas `UNCERTAIN` names no settling task"))
            elif not refs & task_ids:
                issues.append(Issue("error", "blueprint.md", n, f"Gotchas `UNCERTAIN` cites a task that does not exist: {sorted(refs)}"))
    return issues


# --- effectful shell ---------------------------------------------------------


def repo_root(folder: Path) -> Path:
    """Same rule as the CLI. A bare `nabu/` folder is not enough: a tablet can be named `nabu`."""
    for parent in folder.parents:
        if (parent / "nabu/tablets").is_dir() or (parent / "nabu/config.toml").is_file():
            return parent
    return next((p for p in folder.parents if (p / ".jj").is_dir()), folder)


def repo_suffixes(root: Path) -> set[str]:
    """Every repo-relative path and all of its trailing sub-paths."""
    if shutil.which("fd"):
        out = subprocess.run(["fd", "--hidden", "--exclude", ".git", "--exclude", ".jj", ".", str(root)], capture_output=True, text=True).stdout
        paths = [Path(p).relative_to(root).as_posix() for p in out.splitlines()]
    else:
        paths = []
        for p in root.rglob("*"):
            rel = p.relative_to(root)
            if not SKIP_DIRS & set(rel.parts):
                paths.append(rel.as_posix())
    suffixes = set()
    for p in paths:
        parts = p.rstrip("/").split("/")
        for i in range(len(parts)):
            suffixes.add("/".join(parts[i:]))
    return suffixes


def preamble_bases(tasks: str, root: Path) -> list[Path]:
    preamble = tasks.split("\n## ", 1)[0]
    # The parent holds sibling repos a task may cite (`other-repo/...`).
    bases = [root, root.parent]
    for token in BACKTICK_RE.findall(preamble):
        candidate = Path(token) if token.startswith("/") else root / token
        if candidate.is_dir():
            bases.append(candidate)
    return bases


def jj_resolver(root: Path):
    if not shutil.which("jj") or not (root / ".jj").exists():
        return lambda _id: True
    def resolve(change_id: str) -> bool:
        # --ignore-working-copy: read-only, no snapshot of other sessions' edits.
        result = subprocess.run(["jj", "-R", str(root), "--ignore-working-copy", "log", "--no-graph", "-r", change_id, "-T", "''"], capture_output=True)
        return result.returncode == 0
    return resolve


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(__doc__.strip().splitlines()[-2], file=sys.stderr)
        return 2
    folder = Path(argv[1]).resolve()
    tasks_path, blueprint_path = folder / "tasks.md", folder / "blueprint.md"
    # Empty input must never pass the gate.
    if not tasks_path.is_file():
        print(f"error: {tasks_path} does not exist\nfix: pass a plain tablet or one slice folder that holds tasks.md", file=sys.stderr)
        return 2
    tasks = tasks_path.read_text()
    blueprint = blueprint_path.read_text() if blueprint_path.exists() else ""
    root = repo_root(folder)
    issues = [
        *check_paths(tasks, preamble_bases(tasks, root), repo_suffixes(root)),
        *check_workdir(tasks),
        *check_change_ids(tasks, blueprint, jj_resolver(root)),
        *check_uncertain(blueprint, set(TASK_RE.findall(tasks))),
    ]
    for issue in issues:
        print(f"{issue.level}: {issue.file}:{issue.line}: {issue.message}")
    errors = sum(i.level == "error" for i in issues)
    print(f"{errors} error(s), {len(issues) - errors} warning(s)")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
