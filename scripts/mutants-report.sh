#!/usr/bin/env bash
set -euo pipefail

code=${CODE:-0}
title=${TITLE:-mutants}
out=${OUT:-mutants.out}

count() {
  if [ -s "$1" ]; then grep -c . "$1"; else echo 0; fi
}

caught=$(count "$out/caught.txt")
missed=$(count "$out/missed.txt")
timed=$(count "$out/timeout.txt")
unviable=$(count "$out/unviable.txt")

{
  echo "### $title"
  echo
  echo "| caught | survived | timed out | unviable |"
  echo "| ---: | ---: | ---: | ---: |"
  echo "| $caught | $missed | $timed | $unviable |"
  echo
  if [ "$missed" -gt 0 ]; then
    echo "A survivor is a line the tests are not watching."
    echo
    echo '```'
    cat "$out/missed.txt"
    echo '```'
  else
    echo "Nothing survived."
  fi
} >> "${GITHUB_STEP_SUMMARY:-/dev/stdout}"

case "$code" in
  0 | 2 | 3)
    exit 0
    ;;
  4)
    echo "::error::the tests already fail unmutated, so nothing above means anything"
    exit 1
    ;;
  5 | 6)
    echo "::error::the diff handed to --in-diff does not describe this tree"
    exit 1
    ;;
  *)
    echo "::error::cargo mutants stopped with $code"
    exit 1
    ;;
esac
