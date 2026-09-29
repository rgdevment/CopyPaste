#!/usr/bin/env bash
set -euo pipefail

manifest=${1:-}
sig=${2:-}
expected=${3:-}

if [ -z "$manifest" ] || [ -z "$sig" ] || [ -z "$expected" ]; then
  echo "usage: verify_manifest.sh <manifest.json> <manifest.json.sig> <pubkey.txt>" >&2
  exit 64
fi

for one in "$manifest" "$sig" "$expected"; do
  if [ ! -s "$one" ]; then
    echo "::error::verify_manifest: $one is missing or empty"
    exit 66
  fi
done

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

theirs=$(tr -d '[:space:]' < "$expected")
if ! printf '%s' "$theirs" | base64 -d > "$work/pub.bin" 2>/dev/null \
   || [ "$(wc -c < "$work/pub.bin")" -ne 32 ]; then
  echo "::error::verify_manifest: $expected is not a 32-byte Ed25519 key in base64"
  exit 65
fi

if ! tr -d '[:space:]' < "$sig" | base64 -d > "$work/raw.sig" 2>/dev/null \
   || [ "$(wc -c < "$work/raw.sig")" -ne 64 ]; then
  echo "::error::verify_manifest: $sig is not a 64-byte Ed25519 signature in base64"
  exit 65
fi

spki_ed25519='\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00'
{ printf "$spki_ed25519"; cat "$work/pub.bin"; } > "$work/pub.der"
openssl pkey -pubin -inform DER -in "$work/pub.der" -out "$work/pub.pem"

if ! openssl pkeyutl -verify -rawin -pubin -inkey "$work/pub.pem" \
     -in "$manifest" -sigfile "$work/raw.sig" > /dev/null; then
  echo "::error::verify_manifest: $manifest does not answer to $theirs, and a 2.x would drop it \
without a word"
  exit 1
fi

echo "$manifest verifies against $theirs"
