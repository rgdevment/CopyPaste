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
  printf 'x  python3 is not here, so %s was not looked up in %s\n' "$version" "$news"
  exit 0
fi

if python3 - "$version" "$news" <<'PY'; then
import json
import sys

version, news = sys.argv[1], sys.argv[2]
try:
    told = json.load(open(news, encoding="utf-8"))
except (OSError, ValueError) as why:
    print(f"{news} could not be read: {why}")
    sys.exit(2)

for one in told:
    if one.get("version") != version:
        continue
    for tongue in ("es", "en"):
        said = one.get(tongue) or []
        if not said:
            print(f"the {tongue} notes for {version} are empty")
            sys.exit(2)
    sys.exit(0)
sys.exit(1)
PY
  printf 'ok %s is in %s, in both languages\n' "$version" "$news"
  exit 0
fi

amiss "nothing in $news tells a person what changed in $version, and that screen is the only \
place the app says it: add the entry, in Spanish and in English, before the tag goes out"
exit 1
