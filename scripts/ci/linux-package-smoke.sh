#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <deb|rpm> <package-path>" >&2
}

if [[ $# -ne 2 ]]; then
  usage
  exit 1
fi

package_type="$1"
package_path="$2"

if [[ "$package_path" = /* ]]; then
  host_package="$package_path"
else
  host_package="$PWD/$package_path"
fi

if [[ ! -f "$host_package" ]]; then
  echo "Package file not found: $host_package" >&2
  exit 1
fi

if [[ "$package_type" == "deb" ]]; then
  docker run --rm -v "$host_package:/tmp/copypaste.deb:ro" ubuntu:22.04 bash -lc '
    set -euo pipefail
    apt-get update
    DEBIAN_FRONTEND=noninteractive apt-get install -y file /tmp/copypaste.deb

    BIN=$(command -v copypaste || true)
    if [[ -z "$BIN" ]]; then
      BIN=$(find /usr -type f -name copypaste 2>/dev/null | head -n 1)
    fi
    test -n "$BIN"

    mapfile -t ELF_FILES < <(
      dpkg -L copypaste | while read -r path; do
        [[ -f "$path" ]] || continue
        if file -b "$path" | grep -Eq "ELF .* (executable|shared object)"; then
          echo "$path"
        fi
      done
    )

    test "${#ELF_FILES[@]}" -gt 0
    missing=0
    for elf in "${ELF_FILES[@]}"; do
      echo "Checking ELF deps: $elf"
      if ldd "$elf" 2>/dev/null | tee -a /tmp/ldd.out | grep -q "not found"; then
        missing=1
      fi
    done

    test "$missing" -eq 0
  '
elif [[ "$package_type" == "rpm" ]]; then
  docker run --rm -v "$host_package:/tmp/copypaste.rpm:ro" fedora:40 bash -lc '
    set -euo pipefail
    dnf -y install file /tmp/copypaste.rpm

    BIN=$(command -v copypaste || true)
    if [[ -z "$BIN" ]]; then
      BIN=$(find /usr -type f -name copypaste 2>/dev/null | head -n 1)
    fi
    test -n "$BIN"

    mapfile -t ELF_FILES < <(
      rpm -ql copypaste | while read -r path; do
        [[ -f "$path" ]] || continue
        if file -b "$path" | grep -Eq "ELF .* (executable|shared object)"; then
          echo "$path"
        fi
      done
    )

    test "${#ELF_FILES[@]}" -gt 0
    missing=0
    for elf in "${ELF_FILES[@]}"; do
      echo "Checking ELF deps: $elf"
      if ldd "$elf" 2>/dev/null | tee -a /tmp/ldd.out | grep -q "not found"; then
        missing=1
      fi
    done

    test "$missing" -eq 0
  '
else
  usage
  exit 1
fi
