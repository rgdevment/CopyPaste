#!/usr/bin/env bash
set -euo pipefail

file=${1:-}
sig=${2:-}
retired=${3:-}

if [ -z "$file" ] || [ -z "$sig" ]; then
  echo "usage: verify_update.sh <file> <file.minisig> [retired.pub]" >&2
  exit 64
fi

for one in "$file" "$sig" ${retired:+"$retired"}; do
  if [ ! -s "$one" ]; then
    echo "::error::verify_update: $one is missing or empty" >&2
    exit 66
  fi
done

root="$(cd "$(dirname "$0")/.." && pwd)"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

jq -r '.plugins.updater.pubkey' "$root/app/src-tauri/tauri.conf.json" | tr -d '[:space:]' | base64 -d > "$work/primary.pub"
tr -d '[:space:]' < "$root/app/src-tauri/updater-spare.pub" | base64 -d > "$work/spare.pub"
keys="primary spare"
if [ -n "$retired" ]; then
  tr -d '[:space:]' < "$retired" | base64 -d > "$work/retired.pub"
  keys="$keys retired"
fi

for key in $keys; do
  if minisign -Vm "$file" -p "$work/$key.pub" -x "$sig" > /dev/null 2>&1; then
    echo "$key"
    exit 0
  fi
done

echo "::error::verify_update: $file answers to none of: $keys" >&2
exit 1
