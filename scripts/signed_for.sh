#!/usr/bin/env bash
set -uo pipefail

sig=${1:?the decoded minisign signature}
version=${2:?the version it has to be signed for}

comment=$(grep -m1 '^trusted comment: ' "$sig" | tr -d '\r')
fields=$(printf '%s\n' "${comment#trusted comment: }" | tr '\t' '\n')

if printf '%s\n' "$fields" | grep -qxF "version:$version"; then
  exit 0
fi
if printf '%s\n' "$fields" | grep -q '^version:'; then
  echo "signed for $(printf '%s\n' "$fields" | sed -n 's/^version://p' | head -1), not $version" >&2
  exit 1
fi
echo "signed for no version at all" >&2
exit 2
