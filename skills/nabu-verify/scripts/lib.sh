#!/usr/bin/env bash

set -euo pipefail

# Prefer jj when both exist (colocated checkouts).
detect_vcs() {
  if jj root >/dev/null 2>&1; then
    printf 'jj\n'
  elif git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    printf 'git\n'
  else
    echo "error: not inside a jj or git repository" >&2
    exit 1
  fi
}

# Default path noise excluded from scans (tracked but high-churn).
# Override / extend with DIFF_SCAN_EXCLUDE (colon-separated patterns).
diff_scan_default_exclude_patterns() {
  printf '%s\n' \
    'nabu/**' \
    '**/pnpm-lock.yaml'
}

diff_scan_exclude_patterns() {
  diff_scan_default_exclude_patterns
  if [[ -n ${DIFF_SCAN_EXCLUDE:-} ]]; then
    local IFS=':'
    local -a extra
    read -r -a extra <<<"$DIFF_SCAN_EXCLUDE"
    local pat
    for pat in "${extra[@]}"; do
      [[ -z "$pat" ]] && continue
      printf '%s\n' "$pat"
    done
  fi
}

diff_scan_jj_fileset() {
  local -a pats=()
  local pat
  while IFS= read -r pat; do
    [[ -z "$pat" ]] && continue
    pats+=("$pat")
  done < <(diff_scan_exclude_patterns)

  if [[ ${#pats[@]} -eq 0 ]]; then
    printf 'all()\n'
    return 0
  fi

  local union="${pats[0]}"
  local i
  for ((i = 1; i < ${#pats[@]}; i++)); do
    union+=" | ${pats[i]}"
  done

  printf '~(%s)\n' "$union"
}

diff_scan_git_pathspecs() {
  printf '%s\n' '.'
  local pat
  while IFS= read -r pat; do
    [[ -z "$pat" ]] && continue
    printf ':(exclude,glob)%s\n' "$pat"
  done < <(diff_scan_exclude_patterns)
}

# Names list_full_diff drops: lockfiles always, test trees unless include_tests.
# Nabu paths stay (spec edits = review surface). Test match mirrors
# parse_added_lines_from_diff's is_test_path: keep the two in sync.
full_diff_drop_re() {
  local include_tests="${1:-0}"
  local lock='(^|/)(pnpm-lock\.yaml|packages\.lock\.json)$'
  local tests='(^|/)(__tests__|[Tt]ests?)/|\.[Tt]ests?/|[^/]+\.(test|spec)\.[^/]+$|[^/]+Tests?\.cs$'
  if [[ "$include_tests" == 1 ]]; then
    printf '%s' "($lock)"
  else
    printf '%s' "($lock|$tests)"
  fi
}

# Scan entry-point flag parsing shared by list_full_diff / list_added_lines.
# INCLUDE_TESTS=1 sets default, explicit flags win. Prints 0|1.
# $1 = calling fn name (for errors), rest = its args.
resolve_include_tests() {
  local who="$1"
  shift
  local include_tests=0
  if [[ "${INCLUDE_TESTS:-0}" == "1" ]]; then
    include_tests=1
  fi
  local arg
  for arg in "$@"; do
    case "$arg" in
      --include-tests) include_tests=1 ;;
      --exclude-tests) include_tests=0 ;;
      *)
        echo "error: unknown argument to $who: $arg (want --include-tests|--exclude-tests)" >&2
        return 1
        ;;
    esac
  done
  printf '%s\n' "$include_tests"
}

# Diff base for REF (merge target). Fork point of REF and the working copy, not
# REF's tip: once the target moves ahead, a tip diff shows its newer commits as
# reversed edits. Fork point = PR preview (branch's own changes only).
diff_base() {
  case "$1" in
    jj) printf 'fork_point((%s) | @)\n' "$REF" ;;
    git) git merge-base "$REF" HEAD ;;
  esac
}

# Git-format diff of non-test, non-lockfile paths. Args: --include-tests|--exclude-tests.
# Env: REF — merge target; diff from its fork point. Tests off by default (diff noise), but test
# deletions are review surface for test/doc reviewers -> opt in.
# Path args reach jj as a bash argv array, one quoted fileset per name.
# jj parses path args as filesets -> literal quotes stop () in Next.js route
# groups from being fileset syntax. Array argv keeps those quotes; no xargs quote
# processing to defeat. Empty name set -> return 0, never an unrestricted diff.
list_full_diff() {
  local include_tests
  include_tests="$(resolve_include_tests list_full_diff "$@")"

  local vcs
  vcs="$(detect_vcs)"

  local -a name_cmd=()
  local -a diff_cmd=()
  case "$vcs" in
    jj)
      name_cmd=(jj diff --name-only)
      diff_cmd=(jj diff --git -b)
      if [[ -n "${REF:-}" ]]; then
        local base
        base="$(diff_base jj)"
        name_cmd+=(--from "$base")
        diff_cmd+=(--from "$base")
      fi
      ;;
    git)
      name_cmd=(git diff --name-only)
      diff_cmd=(git diff --ignore-all-space)
      if [[ -n "${REF:-}" ]]; then
        local base
        base="$(diff_base git)"
        name_cmd+=("$base")
        diff_cmd+=("$base")
      else
        name_cmd+=(HEAD)
        diff_cmd+=(HEAD)
      fi
      ;;
  esac

  local -a names=()
  local n
  # rg exits 1 when everything filters out -> swallow, empty set handled below
  while IFS= read -r n; do
    [[ -z "$n" ]] && continue
    names+=("$n")
  done < <("${name_cmd[@]}" | rg -v "$(full_diff_drop_re "$include_tests")" || true)

  [[ ${#names[@]} -eq 0 ]] && return 0

  local -a paths=()
  if [[ "$vcs" == jj ]]; then
    for n in "${names[@]}"; do
      paths+=("\"${n//\"/\\\"}\"")
    done
  else
    paths=("${names[@]}")
  fi

  "${diff_cmd[@]}" -- "${paths[@]}"
}

# Emits compact added-line stream (path once per file, not per line):
#   path
#   line<TAB>content
#   line<TAB>content
# Why: repeating path:line on every row balloons tokens when agents ingest the scan.
#
# Tests default OFF (~80% of added-line volume in typical changes). Opt in with
# `--include-tests` / `--exclude-tests`, or INCLUDE_TESTS=1. Explicit flags win
# over the env var. Optional env: REF — merge target; diff from its fork point.
list_added_lines() {
  local include_tests
  include_tests="$(resolve_include_tests list_added_lines "$@")"

  local vcs
  vcs="$(detect_vcs)"

  case "$vcs" in
    jj)
      local fileset
      fileset="$(diff_scan_jj_fileset)"
      local diff_cmd=(jj diff --git -b)
      if [[ -n "${REF:-}" ]]; then
        diff_cmd+=(--from "$(diff_base jj)")
      fi
      diff_cmd+=(-- "$fileset")
      "${diff_cmd[@]}" | parse_added_lines_from_diff "$include_tests"
      ;;
    git)
      local -a pathspecs=()
      local ps
      while IFS= read -r ps; do
        [[ -z "$ps" ]] && continue
        pathspecs+=("$ps")
      done < <(diff_scan_git_pathspecs)
      local diff_cmd=(git diff --ignore-all-space)
      if [[ -n "${REF:-}" ]]; then
        diff_cmd+=("$(diff_base git)")
      else
        diff_cmd+=(HEAD)
      fi
      diff_cmd+=(-- "${pathspecs[@]}")
      "${diff_cmd[@]}" | parse_added_lines_from_diff "$include_tests"
      ;;
  esac
}

# $1: 1 = emit test paths, 0 = skip test paths (same match as full_diff_drop_re)
parse_added_lines_from_diff() {
  local include_tests="${1:-0}"
  awk -v include_tests="$include_tests" '
    function is_test_path(f) {
      return f ~ /(^|\/)(__tests__|[Tt]ests?)\// || f ~ /\.[Tt]ests?\// || f ~ /[^\/]+\.(test|spec)\.[^\/]+$/ || f ~ /[^\/]+Tests?\.cs$/
    }

    BEGIN {
      file = ""
      line = 0
      need_header = 0
      skip = 0
    }

    /^\+\+\+ b\// {
      file = substr($0, 7)
      skip = (include_tests == 0 && is_test_path(file))
      need_header = (skip == 0)
      next
    }

    /^@@ / {
      if (match($0, /\+([0-9]+)/)) {
        line = substr($0, RSTART + 1, RLENGTH - 1) + 0
      }
      next
    }

    /^\+/ && $0 !~ /^\+\+\+/ {
      if (!skip) {
        if (need_header) {
          printf "%s\n", file
          need_header = 0
        }
        printf "%d\t%s\n", line, substr($0, 2)
      }
      line += 1
      next
    }

    /^-/ {
      next
    }

    {
      line += 1
    }
  '
}
