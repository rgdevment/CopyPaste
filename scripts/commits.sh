#!/usr/bin/env bash
set -uo pipefail

shape='^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([a-z0-9._-]+\))?: .+'
elsewhere='https?://[^[:space:]/]+\.|(^|[[:space:](<])www\.|(^|[[:space:](<])([[:alnum:]-]+\.)+[[:alpha:]]{2,}/|[[:alnum:]._-]+@[[:alnum:].-]+:|[[:alnum:]_.-]+/[[:alnum:]_.-]+#[0-9]+'
most=120
status=0

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
  status=1
}

# commit-msg runs on git merge too, where a refusal leaves it half done
git_wrote() {
  case $1 in
    "Merge branch '"* | "Merge branches "* | "Merge pull request #"* | \
      "Merge remote-tracking branch '"* | "Merge commit '"* | "Merge tag '"* | \
      "Merge http://"* | "Merge https://"* | "Merge git://"* | "Merge ssh://"* | \
      "Squashed commit of the following:"* | 'Revert "'* | 'Reapply "'* | \
      "fixup! "* | "squash! "* | "amend! "*)
      return 0
      ;;
  esac
  return 1
}

trimmed() {
  printf '%s' "$1" | LC_ALL=C sed 's/^[[:space:]]*//;s/[[:space:]]*$//'
}

# strict is for a title, which reaches main: only what git writes itself keeps its shape there
weighed() {
  local who=$1 said strict=${3:-}
  said=$(trimmed "$2")
  if [ -z "$strict" ] && git_wrote "$said"; then
    return
  fi
  local broke=""
  case $said in
    'Revert "'* | 'Reapply "'*) ;;
    *)
      if ! printf '%s' "$said" | grep -qE "$shape"; then
        broke=1
        amiss "$who does not follow the convention"
      fi
      ;;
  esac
  if printf '%s' "$said" | grep -qiE "$elsewhere"; then
    broke=1
    amiss "$who links outside this repository; only #123 of this one"
  fi
  # bytes minus UTF-8 continuation bytes: ${#said} counts bytes when a client spawns git with no locale
  local long
  long=$(printf '%s' "$said" | LC_ALL=C tr -d '\200-\277' | wc -c | tr -d ' ')
  if [ "$long" -gt "$most" ]; then
    broke=1
    amiss "$who is $long characters, and the subject goes under $most"
  fi
  [ -z "$broke" ] || printf '  %s\n' "$said"
}

bodiless() {
  local who=$1 subject body
  subject=$(trimmed "$2")
  body=$(trimmed "$3")
  if [ -n "$body" ] && ! git_wrote "$subject"; then
    amiss "$who carries more than its subject line: a body, a trailer or a wrapped subject"
    printf '  %s\n' "$subject"
  fi
}

how_it_reads() {
  if [ "$status" -eq 1 ]; then
    printf '\n'
    printf 'Expected: type(optional scope): concrete change, one line, in English\n'
    printf 'Types:    feat fix docs style refactor perf test build ci chore revert\n'
    printf 'Example:  fix(panel): the menu yields to the wheel\n'
  fi
  exit $status
}

case ${1:-} in
  --title)
    [ -n "${2:-}" ] || {
      echo "usage: commits.sh --title <text>"
      exit 2
    }
    weighed "the title" "$2" strict
    [ "$status" -eq 0 ] && [ -z "${GITHUB_ACTIONS:-}" ] && printf 'ok the title is well formed\n'
    how_it_reads
    ;;
  --message)
    [ -f "${2:-}" ] || {
      echo "usage: commits.sh --message <file>"
      exit 2
    }
    # git strips its own comment lines after this hook runs, and only when an editor wrote them
    mark=$(git config core.commentChar 2> /dev/null || echo '#')
    case $mark in auto | "") mark='#' ;; esac
    said=$(awk -v m="$mark" '
      index($0, m " ") == 1 && $0 ~ / -+ >8 -+$/ { exit }
      $0 == m || index($0, m " ") == 1 || index($0, m "\t") == 1 { next }
      { print }
    ' "$2" | git stripspace)
    [ -n "$said" ] || exit 0
    subject=$(printf '%s\n' "$said" | head -1)
    weighed "the subject" "$subject"
    bodiless "the message" "$subject" "$(printf '%s\n' "$said" | tail -n +2)"
    [ "$status" -eq 0 ] && [ -z "${GITHUB_ACTIONS:-}" ] && printf 'ok the message is well formed\n'
    how_it_reads
    ;;
esac

[ $# -gt 0 ] || {
  echo "usage: commits.sh <revisions…> | --title <text> | --message <file>"
  exit 2
}

# git prints hints on stderr while exiting 0, and folding them in would weigh them as messages
trouble=$(mktemp)
if ! listed=$(git log --no-merges --format='%h%x1f%an%x1f%B%x1e' "$@" 2> "$trouble"); then
  cat "$trouble" >&2
  rm -f "$trouble"
  amiss "the commits in $* could not be listed, so no message was looked at"
  exit 1
fi
rm -f "$trouble"

seen=0
while IFS=$'\x1f' read -r -d $'\x1e' sha name message; do
  sha=$(trimmed "$sha")
  [ -n "$sha" ] || continue
  seen=$((seen + 1))
  subject=$(printf '%s\n' "$message" | head -1)
  weighed "$sha" "$subject"
  # a bot writes its own body with no way to leave it out; its squash title is what reaches main
  case $name in
    *"[bot]") ;;
    *) bodiless "$sha" "$subject" "$(printf '%s\n' "$message" | tail -n +2)" ;;
  esac
done <<< "$listed"

[ "$status" -eq 0 ] && [ -z "${GITHUB_ACTIONS:-}" ] && printf 'ok %s commit message(s) well formed\n' "$seen"
how_it_reads
