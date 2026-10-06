#!/usr/bin/env bash
set -uo pipefail

cd "$(dirname "$0")/.." || exit 2

version=${1:?a version is needed, without the leading v}
news=app/src/news.json

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
}

# the window only shows entries at or below the version running, and 3.0.0 is above 3.0.0-rc1,
# so a candidate would show nothing however carefully its notes were written
case $version in
  *-*)
    printf 'ok %s is a candidate, and the window shows no notes for one\n' "$version"
    exit 0
    ;;
esac

if [ ! -f "$news" ]; then
  amiss "$news is not there, so what the window would show is anybody's guess"
  exit 1
fi

if ! command -v python3 > /dev/null; then
  # a gate that cannot look has to say so: in CI that is a failure, on a machine it is a warning,
  # because a hook that blocks over a missing tool is a hook somebody uninstalls
  amiss "python3 is not here, so nobody looked at what $news says about $version"
  [ -n "${GITHUB_ACTIONS:-}" ] && exit 1
  exit 0
fi

reads='
import json
import sys

version, news = sys.argv[1], sys.argv[2]
try:
    told = json.load(open(news, encoding="utf-8"))
except (OSError, ValueError) as why:
    print(f"{news} could not be read: {why}")
    sys.exit(2)

if not isinstance(told, list):
    print(f"{news} is not the list of versions the window reads")
    sys.exit(2)

for one in told:
    if not isinstance(one, dict) or one.get("version") != version:
        continue
    for tongue in ("es", "en"):
        said = one.get(tongue) or []
        if not said:
            print(f"{version} is in {news} with nothing written in {tongue}")
            sys.exit(2)
    sys.exit(0)
sys.exit(1)
'

said=$(python3 -c "$reads" "$version" "$news" 2>&1)
looked=$?

# a patch fixes what its minor release shipped, and the window already told that release's news
minor="${version%.*}.0"
if [ "$looked" = 1 ] && [ "$minor" != "$version" ] \
  && python3 -c "$reads" "$minor" "$news" > /dev/null 2>&1; then
  printf 'ok %s is a patch of %s, whose entry in %s still says what it brings\n' \
    "$version" "$minor" "$news"
  exit 0
fi

case $looked in
  0)
    printf 'ok %s is in %s, in both languages\n' "$version" "$news"
    exit 0
    ;;
  1)
    amiss "nothing in $news tells a person what changed in $version, and that screen is the \
only place the app says it: add the entry, in Spanish and in English, before the tag goes out"
    exit 1
    ;;
  *)
    amiss "${said:-$news could not be looked through for $version}"
    exit 1
    ;;
esac
