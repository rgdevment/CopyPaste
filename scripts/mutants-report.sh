#!/usr/bin/env bash
set -euo pipefail

code=${CODE:-}
title=${TITLE:-mutants}
out=${OUT:-mutants.out}

# the run step answers with set +e and always writes code, so an empty one never ran
if [ -z "$code" ]; then
  echo "::error::cargo mutants never reported an exit, so it did not run"
  exit 1
fi

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

# a survivor (2) and a timeout (3) are a trend, not a blocker; 4 to 6 mean the measurement is worthless
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
