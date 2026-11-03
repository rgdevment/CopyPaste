#!/usr/bin/env bash
set -uo pipefail

kinds='feat|fix|chore|docs|refactor|test|perf|build|ci|style|revert|hotfix|release'
shape="^($kinds)/[a-z0-9][a-z0-9._/-]*$"
status=0

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
  status=1
}

# A branch says what kind of change it carries, the way its commits do; main and a detached head say nothing.
for name in "$@"; do
  case $name in
    main | HEAD | "") continue ;;
  esac
  if ! printf '%s' "$name" | grep -qE "$shape"; then
    amiss "the branch $name is not named kind/what, with kind one of: ${kinds//|/, }"
  fi
done

exit $status
