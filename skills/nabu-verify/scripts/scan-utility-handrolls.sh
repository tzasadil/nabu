#!/usr/bin/env bash
# Mandatory PATTERN_CONSISTENCY / utility-library scan for structure-agent.
# Lists every introduced new Set / Array.from / for / forEach / foreach in the diff.
# Walk each hit and decide whether it hand-rolls a utility-library helper or LINQ.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# Dynamic path: shellcheck cannot follow unless invoked with -x.
# shellcheck disable=SC1091
source "$SCRIPT_DIR/lib.sh"

# Pattern rules live in scan-utility-handrolls.py (same shape as pr-preparation risk-scan).
list_added_lines | python3 "$SCRIPT_DIR/scan-utility-handrolls.py"
