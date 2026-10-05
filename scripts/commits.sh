#!/usr/bin/env bash
set -uo pipefail

shape='^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([a-z0-9._-]+\))?: .+'
elsewhere='https?://|[[:alnum:]_.-]+/[[:alnum:]_.-]+#[0-9]+'
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
  case $said in
    'Revert "'* | 'Reapply "'*) ;;
    *)
      if ! printf '%s' "$said" | grep -qE "$shape"; then
        amiss "$who does not follow the convention"
        printf '  %s\n' "$said"
        return
      fi
      ;;
  esac
  if printf '%s' "${said% (#[0-9]*)}" | grep -qE "$elsewhere"; then
    amiss "$who links outside this repository; only #123 of this one"
    printf '  %s\n' "$said"
  fi
  # bytes minus UTF-8 continuation bytes: ${#said} counts bytes when a client spawns git with no locale
  local long
  long=$(printf '%s' "$said" | LC_ALL=C tr -d '\200-\277' | wc -c | tr -d ' ')
  if [ "$long" -gt "$most" ]; then
    amiss "$who is $long characters, and the subject goes under $most"
    printf '  %s\n' "$said"
  fi
}

bodiless() {
  local who=$1 subject body
  subject=$(trimmed "$2")
  body=$(trimmed "$3")
  if [ -n "$body" ] && ! git_wrote "$subject"; then
    amiss "$who carries a body or a trailer; a commit is its subject line alone"
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
  --subject | --title)
    [ -n "${2:-}" ] || {
      echo "usage: commits.sh --subject|--title <text>"
      exit 2
    }
    weighed "the subject" "$2" "$([ "$1" = "--title" ] && echo strict)"
    [ "$status" -eq 0 ] && [ -z "${GITHUB_ACTIONS:-}" ] && printf 'ok the subject is well formed\n'
    how_it_reads
    ;;
  --message)
    [ -f "${2:-}" ] || {
      echo "usage: commits.sh --message <file>"
      exit 2
    }
    said=$(sed '/^# -* >8 -*$/,$d' "$2" | git stripspace --strip-comments)
    [ -n "$said" ] || exit 0
    subject=$(printf '%s\n' "$said" | head -1)
    weighed "the subject" "$subject"
    bodiless "the message" "$subject" "$(printf '%s\n' "$said" | tail -n +2)"
    [ "$status" -eq 0 ] && [ -z "${GITHUB_ACTIONS:-}" ] && printf 'ok the message is well formed\n'
    how_it_reads
    ;;
esac

range=${1:-}
[ -n "$range" ] || {
  echo "usage: commits.sh <range> | --subject <text> | --title <text> | --message <file>"
  exit 2
}

# git prints hints on stderr while exiting 0, and folding them in would weigh them as subjects
trouble=$(mktemp)
if ! listed=$(git rev-list --no-merges "$range" 2> "$trouble"); then
  cat "$trouble" >&2
  rm -f "$trouble"
  amiss "the commits between $range could not be listed, so no subject was looked at"
  exit 1
fi
rm -f "$trouble"

seen=0
while read -r sha; do
  [ -n "$sha" ] || continue
  seen=$((seen + 1))
  subject=$(git log -1 --format=%s "$sha")
  weighed "${sha:0:8}" "$subject"
  bodiless "${sha:0:8}" "$subject" "$(git log -1 --format=%b "$sha")"
done <<< "$listed"

[ "$status" -eq 0 ] && [ -z "${GITHUB_ACTIONS:-}" ] && printf 'ok %s commit subject(s) well formed\n' "$seen"
how_it_reads
