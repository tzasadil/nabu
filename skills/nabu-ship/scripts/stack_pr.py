#!/usr/bin/env python3
"""Open stacked GitHub PRs for a range of jj revisions, bottom-up: one bookmark and one PR per revision.

Each PR's base is the bookmark of the revision below it, the bottom PR's base is --target.
The PR body is the `## Feature description` of the blueprint.md the revision adds, else its description.

Usage: stack_pr.py -t <target> (-s <start> | -e <end> | both) [--dry-run]
  -t  base bookmark of the bottom PR (`main`, `main@origin`, or a revision). Required: no safe default.
  -s  bottom revision. Missing -> the first revision after the fork point of target and end.
  -e  top revision. Missing -> the single head of start's descendants.
  --dry-run  print the bookmarks and the PR chain. Fetches, but pushes and creates nothing.

Needs `jj` and `gh` (logged in). Rerun after edits to push the restacked revisions: existing PRs are reused.
Exit 1 with `error:` and `fix:` on stderr. Every check runs before the first push => a bad range pushes nothing.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys

# Shared bases. A PR head on one of them would move trunk.
PROTECTED = {"main", "master", "develop", "staging", "qa", "prod", "experimental", "develop-experimental"}
# GitHub's pre-receive hook rejects very long ref names.
SLUG_MAX = 60
FEATURE_HEADING = "## Feature description"


class Fail(Exception):
    def __init__(self, error: str, fix: str = ""):
        super().__init__(error)
        self.error, self.fix = error, fix


def run(*cmd: str) -> str:
    done = subprocess.run(cmd, capture_output=True, text=True)
    if done.returncode != 0:
        raise Fail(f"`{' '.join(cmd)}` failed: {done.stderr.strip() or done.stdout.strip()}")
    return done.stdout


def jj(*args: str) -> str:
    return run("jj", "--no-pager", *args)


def lines(text: str) -> list[str]:
    return [line.strip() for line in text.splitlines() if line.strip()]


def slug(raw: str) -> str:
    out = re.sub(r"[^a-z0-9-]", "-", raw.lower())
    return re.sub(r"-+", "-", out)[:SLUG_MAX].strip("-")


def title(rev: str) -> str:
    return jj("log", "-r", rev, "--no-graph", "-T", "description.first_line()").strip()


def bookmarks_on(rev: str) -> list[str]:
    # Local and remote bookmarks print the same name => dedupe.
    return sorted(set(lines(jj("bookmark", "list", "-r", rev, "-T", 'name ++ "\\n"'))))


def head_bookmark(rev: str) -> str:
    """Reuse the one non-protected bookmark on rev => a reworded description keeps its branch and PR."""
    existing = [b for b in bookmarks_on(rev) if b not in PROTECTED]
    if len(existing) > 1:
        raise Fail(f"{rev} has several bookmarks: {', '.join(existing)}", "jj bookmark delete all but one of them")
    name = existing[0] if existing else slug(title(rev))
    if not name:
        raise Fail(f"{rev} has no description", f'jj describe -r {rev} -m "<title>"')
    # A description such as "Main" slugs to a trunk name.
    if name in PROTECTED:
        raise Fail(f"{rev} resolves to protected bookmark '{name}'", f"jj describe -r {rev} with another title")
    return name


def feature_description(blueprint: str) -> str:
    """Body of `## Feature description`, up to the next level-1 or level-2 heading."""
    out, inside = [], False
    for line in blueprint.splitlines():
        if re.match(r"^##? ", line):
            inside = line.rstrip() == FEATURE_HEADING
            continue
        if inside:
            out.append(line)
    return "\n".join(out).strip()


def pr_body(rev: str) -> str:
    # A nabu slice revision adds its blueprint.md, whose feature description explains the PR better than the title.
    added = jj(
        "log", "-r", rev, "--no-graph", "-T",
        'self.diff().files().filter(|f| f.status() == "added").map(|f| f.path() ++ "\\n").join("")',
    )
    blueprints = [p for p in lines(added) if p == "blueprint.md" or p.endswith("/blueprint.md")]
    if blueprints:
        feature = feature_description(jj("file", "show", "-r", rev, f'root-file:"{blueprints[0]}"'))
        if feature:
            return feature
    return jj("log", "-r", rev, "--no-graph", "-T", "description").strip()


def resolve_target(raw: str, dry_run: bool) -> str:
    # `gh pr --base` needs the bare bookmark name. A revset such as `@-` keeps its `@`.
    target = raw if raw.startswith("@") else raw.split("@")[0]
    if not lines(jj("log", "-r", raw, "--no-graph", "--limit", "1", "-T", 'change_id ++ "\\n"')):
        raise Fail(f"cannot resolve target '{raw}'", "pass a bookmark such as main")
    if lines(jj("bookmark", "list", f"exact:{target}")):
        return target
    found = bookmarks_on(raw)
    if len(found) > 1:
        raise Fail(f"target {raw} has several bookmarks: {', '.join(found)}", "pass one of them as -t")
    if found:
        print(f"target: resolved to bookmark '{found[0]}'", file=sys.stderr)
        return found[0]
    name = head_bookmark(raw)
    if dry_run:
        print(f"target: would create and push bookmark '{name}' on {raw}", file=sys.stderr)
        return name
    push_bookmark(name, raw)
    print(f"target: created and pushed bookmark '{name}'", file=sys.stderr)
    return name


def push_bookmark(name: str, rev: str) -> None:
    jj("bookmark", "set", name, "-r", rev)
    jj("git", "push", "--bookmark", name)
    # Already tracked after a first push => ignore the failure.
    subprocess.run(["jj", "bookmark", "track", f"{name}@origin"], capture_output=True)


def stack_range(target: str, start: str | None, end: str | None) -> str:
    if not start and not end:
        raise Fail("no range given", "pass -s <bottom revision> and/or -e <top revision>")
    if not end:
        heads = lines(jj("log", "-r", f"heads({start}::)", "--no-graph", "-T", 'change_id ++ "\\n"'))
        if len(heads) != 1:
            raise Fail(f"-s {start} has {len(heads)} heads, not exactly 1", "pass -e <top revision>")
        end = heads[0]
    if not start:
        # fork_point(...)+ alone also matches fork-point children on other branches => restrict to ancestors of end.
        start = f"roots(fork_point({target} | {end})+::{end})"
    return f"{start}::{end}"


def check_target_on_origin(target: str) -> None:
    if "@origin:" not in jj("bookmark", "list", "--all-remotes", f"exact:{target}"):
        raise Fail(f"target bookmark '{target}' is not on origin", f"jj git push --bookmark {target}")
    if lines(jj("bookmark", "list", "--conflicted", f"exact:{target}")):
        raise Fail(f"target bookmark '{target}' is conflicted", f"jj bookmark set {target} -r {target}@origin")


def check_pushable(target: str, top: str) -> None:
    """`jj git push` refuses a revision without description anywhere below the head => check the whole span first."""
    blank = lines(jj("log", "-r", f"({target}..{top}) & description(exact:'')", "--no-graph", "-T", 'change_id.shortest(8) ++ "\\n"'))
    if blank:
        raise Fail(f"revisions without description between {target} and the stack top: {', '.join(blank)}", f'jj describe -r {blank[0]} -m "<title>"')


def plan(rng: str) -> list[tuple[str, str]]:
    """(change id, head bookmark) bottom-up. Any bad revision aborts before the first push."""
    ids = lines(jj("log", "-r", rng, "--reversed", "--no-graph", "-T", 'change_id ++ "\\n"'))
    if not ids:
        raise Fail(f"no revisions in range {rng}", "check -s and -e")
    out: list[tuple[str, str]] = []
    for rev in ids:
        if not title(rev):
            raise Fail(f"revision {rev} has no description", f'jj describe -r {rev} -m "<title>"')
        name = head_bookmark(rev)
        if any(name == taken for _, taken in out):
            raise Fail(f"two revisions resolve to bookmark '{name}'", "give one of them another title")
        out.append((rev, name))
    return out


def open_prs(target: str, stack: list[tuple[str, str]], dry_run: bool) -> None:
    base = target
    for rev, name in stack:
        print(f"PR: {name} -> {base}")
        if not dry_run:
            # A rerun finds the PR already open: create fails, view still finds the url.
            # TODO: an existing PR keeps its old base. Revisit when a restack changes the slice order.
            created = subprocess.run(
                ["gh", "pr", "create", "--base", base, "--head", name, "--title", title(rev), "--body", pr_body(rev)],
                capture_output=True,
                text=True,
            )
            view = subprocess.run(["gh", "pr", "view", name, "--json", "url", "--jq", ".url"], capture_output=True, text=True)
            if view.returncode != 0:
                raise Fail(f"no PR for '{name}': {created.stderr.strip() or view.stderr.strip()}", "gh auth status")
            print(f"  {view.stdout.strip()}")
        base = name


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("-t", "--target", required=True)
    parser.add_argument("-s", "--start")
    parser.add_argument("-e", "--end")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    try:
        jj("git", "fetch")
        target = resolve_target(args.target, args.dry_run)
        rng = stack_range(target, args.start, args.end)
        stack = plan(rng)
        check_pushable(target, stack[-1][0])
        # A dry run may name a target bookmark that does not exist yet.
        if not args.dry_run or lines(jj("bookmark", "list", f"exact:{target}")):
            check_target_on_origin(target)
        if not args.dry_run:
            for rev, name in stack:
                # The configured author, not whoever first created the revision.
                jj("metaedit", "--update-author", "--force-rewrite", "-r", rev)
                push_bookmark(name, rev)
        open_prs(target, stack, args.dry_run)
    except Fail as fail:
        print(f"error: {fail.error}", file=sys.stderr)
        if fail.fix:
            print(f"fix: {fail.fix}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
