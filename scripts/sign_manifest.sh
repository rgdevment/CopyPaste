#!/usr/bin/env bash
set -euo pipefail

manifest=${1:-}
out=${2:-}
expected=${3:-}

if [ -z "$manifest" ] || [ -z "$out" ] || [ -z "$expected" ]; then
  echo "usage: sign_manifest.sh <manifest.json> <out.sig> <expected-pubkey.txt>" >&2
  echo "the base64 Ed25519 seed is read from stdin" >&2
  exit 64
fi

for one in "$manifest" "$expected"; do
  if [ ! -s "$one" ]; then
    echo "::error::sign_manifest: $one is missing or empty"
    exit 66
  fi
done

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

seed=$(tr -d '[:space:]')
if [ -z "$seed" ]; then
  echo "::error::sign_manifest: no seed on stdin"
  exit 65
fi
if ! printf '%s' "$seed" | base64 -d > "$work/seed.bin" 2>/dev/null; then
  echo "::error::sign_manifest: the seed on stdin is not base64"
  exit 65
fi
held=$(wc -c < "$work/seed.bin")
if [ "$held" -ne 32 ]; then
  echo "::error::sign_manifest: the seed decodes to $held bytes and Ed25519 wants 32"
  exit 65
fi

pkcs8_ed25519='\x30\x2e\x02\x01\x00\x30\x05\x06\x03\x2b\x65\x70\x04\x22\x04\x20'

{ printf "$pkcs8_ed25519"; cat "$work/seed.bin"; } > "$work/key.der"
openssl pkey -inform DER -in "$work/key.der" -out "$work/key.pem"

mine=$(openssl pkey -in "$work/key.pem" -pubout -outform DER | tail -c 32 | base64 -w0)
theirs=$(tr -d '[:space:]' < "$expected")
if [ "$mine" != "$theirs" ]; then
  echo "::error::sign_manifest: the seed answers to $mine and the copies out there carry \
$theirs, so nothing signed with it would ever verify. Nothing was written."
  exit 1
fi

openssl pkeyutl -sign -rawin -inkey "$work/key.pem" -in "$manifest" -out "$work/raw.sig"
base64 -w0 < "$work/raw.sig" > "$work/sig.txt"
printf '\n' >> "$work/sig.txt"

if ! bash "$(dirname "$0")/verify_manifest.sh" "$manifest" "$work/sig.txt" "$expected"; then
  echo "::error::sign_manifest: the signature it just made does not verify. Nothing was written."
  exit 1
fi

cp "$work/sig.txt" "$out"
echo "$manifest is signed, and the signature answers to $theirs"
