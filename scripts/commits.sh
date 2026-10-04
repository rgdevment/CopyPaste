#!/usr/bin/env bash
set -uo pipefail

shape='^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([a-z0-9._-]+\))?!?: .+'
most=90
status=0

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
  status=1
}

spared() {
  case $1 in
    Merge\ * | Revert\ * | fixup!\ * | squash!\ * | amend!\ *) return 0 ;;
  esac
  return 1
}

weighed() {
  local who=$1 said=$2
  said=$(printf '%s' "$said" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
  if spared "$said"; then
    return
  fi
  if ! printf '%s' "$said" | grep -qE "$shape"; then
    amiss "$who does not follow the convention"
    printf '  %s\n' "$said"
    return
  fi
  if [ "${#said}" -gt "$most" ]; then
    amiss "$who is ${#said} characters, and the subject goes under $most"
    printf '  %s\n' "$said"
  fi
}

how_it_reads() {
  printf '\n'
  printf 'Expected: type(optional scope): description\n'
  printf 'Types:    feat fix docs style refactor perf test build ci chore revert\n'
  printf 'Example:  fix(panel): the menu yields to the wheel\n'
}

if [ "${1:-}" = "--subject" ]; then
  weighed "the subject" "${2:-}"
else
  while read -r sha; do
    [ -n "$sha" ] || continue
    weighed "${sha:0:8}" "$(git log -1 --format=%s "$sha")"
  done < <(git rev-list --no-merges "${1:?a range or --subject is needed}")
fi

[ "$status" -eq 0 ] || how_it_reads
exit $status
