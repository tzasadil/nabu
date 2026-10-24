#!/usr/bin/env python3
"""List hand-rolled collection loops on added lines (structure-agent PATTERN_CONSISTENCY).

Reads compact added-line stream from stdin (see list_added_lines):
  path
  line\\tcontent
Same shape as pr-preparation risk-scan.py.
A line may match multiple categories.
"""

from __future__ import annotations

import re
import sys


def main() -> int:
    current_file = ""
    for raw in sys.stdin:
        line = raw.rstrip("\n")
        # Path headers have no tab; content rows are "lineno\\tcontent".
        if "\t" not in line:
            current_file = line
            continue
        if not current_file:
            continue
        lineno, content = line.split("\t", 1)
        location = f"{current_file}:{lineno}"

        # Docs/markdown examples are noise for this scan (location is path:line)
        if re.search(r"\.(md|mdx|txt):\d+$", location):
            continue

        if re.search(r"new\s+Set([^A-Za-z0-9_]|$)", content):
            print(f"[new-Set] {location} {content}")

        if re.search(r"Array\.from([^A-Za-z0-9_]|$)", content):
            print(f"[Array.from] {location} {content}")

        if re.search(r"\.forEach\s*\(", content):
            print(f"[forEach] {location} {content}")

        # for (...), for (const x of/in ...), C-style for (let i = 0; ...)
        if re.search(r"(^|[\s;{}])for\s*\(", content):
            print(f"[for] {location} {content}")

        # C# foreach: `for` regex above needs `for` directly before `(`
        if re.search(r"(^|[\s;{}])foreach\s*\(", content):
            print(f"[foreach] {location} {content}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
