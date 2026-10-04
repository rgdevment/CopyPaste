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
  if [ "$missed" -gt 0 ] || [ "$timed" -gt 0 ]; then
    echo "A survivor is a line the tests are not watching. **This check stays green with"
    echo "survivors on purpose**, so green here means the sweep ran, not that nothing survived."
    echo
  fi
  if [ "$missed" -gt 0 ]; then
    echo '```'
    cat "$out/missed.txt"
    echo '```'
  elif [ "$timed" -gt 0 ]; then
    echo "Nothing survived, and something timed out."
  else
    echo "Nothing survived."
  fi
} >> "${GITHUB_STEP_SUMMARY:-/dev/stdout}"

# Decided 2026-10-04: a survivor (2) and a timeout (3) are reported, not blocking. A mutation
# score is a trend, and one survivor in an unrelated crate should not hold a pull request. What
# must never happen is green being read as «nothing survived», which the summary above now says
# outright. A suite that fails unmutated (4) or a diff that does not describe the tree (5, 6) are
# different: those mean the measurement is worthless, and they fail.
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
