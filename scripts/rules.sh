#!/usr/bin/env bash
set -uo pipefail

status=0

amiss() {
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::error::%s\n' "$1"
  else
    printf 'x  %s\n' "$1"
  fi
  status=1
}

went_well() {
  [ -n "${GITHUB_ACTIONS:-}" ] || printf 'ok %s\n' "$1"
}

# grep answers 1 on nothing found and 2 when it could not look: a rule that cannot look must shout
found_nothing() {
  case $2 in
    0) amiss "$1"; return 1 ;;
    1) return 0 ;;
    *) amiss "$3"; return 1 ;;
  esac
}

unsafe_only_in_the_sys_crates() {
  local expected="crates/cp-mac-sys crates/cp-win-sys" found dir loose=0
  found=$(grep -rl 'unsafe_code = "allow"' crates/*/Cargo.toml | xargs -n1 dirname | sort)
  for dir in $found; do
    case " $expected " in
      *" $dir "*) ;;
      *) amiss "unsafe is declared outside the -sys crates: $dir"; loose=1 ;;
    esac
  done
  [ "$loose" -eq 0 ] && went_well "unsafe lives only in the -sys crates"
}

nothing_the_core_prints() {
  grep -rnE '^[^"]*(^|[ \t{;(=>|&])(e?print(ln)?!)' crates/cp-core/src --include='*.rs'
  found_nothing \
    "cp-core does not print: use tracing" $? \
    "cp-core could not be looked through for what it prints" \
    && went_well "the core produces no terminal output"
}

nothing_the_core_depends_on() {
  grep -rn 'use tauri\|use windows\|use objc2' crates/cp-core/src --include='*.rs'
  found_nothing \
    "cp-core depends on neither platform nor interface" $? \
    "cp-core could not be looked through for what it depends on" \
    && went_well "the core touches neither platform nor interface"
}

written_in_english() {
  grep -rnE '\b(fecha|limite|prioridad|filtro|tarea|titulo|etiqueta|imagen|archivo) *:' \
    crates/*/src --include='*.rs' | grep -v '"' | grep -vE ':[0-9]+: *///?!?'
  found_nothing \
    "identifiers are written in English" $? \
    "the identifiers could not be looked through" \
    && went_well "no Spanish identifiers"
}

no_comments_in_the_code() {
  grep -rnE '[/]{2}' crates app/src-tauri/src --include='*.rs' \
    | grep -vE '[a-z]+:[/][/]' | grep -vE '"[^"]*[/]{2}'
  found_nothing \
    "the code carries no comments; whatever needs explaining goes on the record" $? \
    "the code could not be looked through for comments" \
    && went_well "the code carries no comments"
  grep -rnE '[/][*]' crates app/src-tauri/src --include='*.rs'
  found_nothing \
    "no block comments either" $? \
    "the code could not be looked through for block comments" \
    && went_well "no block comments either"
}

# cargo links every example to target/<profile>/examples/<name>, so two of a name race
every_example_has_its_own_name() {
  local dirs said twice one
  dirs=$(find . -type d -name examples -not -path './target/*' -not -path '*/node_modules/*' | sort)
  said=$(for one in $dirs; do
    find "$one" -mindepth 1 -maxdepth 1 -name '*.rs' -exec basename {} .rs \;
    find "$one" -mindepth 2 -maxdepth 2 -name 'main.rs' -print \
      | xargs -r -n1 dirname | xargs -r -n1 basename
  done | sort)
  if [ -z "$said" ]; then
    amiss "no example was found, so nobody saw whether two share a name"
    return
  fi
  twice=$(printf '%s\n' "$said" | uniq -d)
  if [ -n "$twice" ]; then
    amiss "two crates name an example the same: $(printf '%s' "$twice" | tr '\n' ' ')"
  else
    went_well "every example has a name of its own"
  fi
}

what_python_measures() {
  local why=$1 said
  shift
  if ! command -v python3 > /dev/null; then
    amiss "python3 is not here, so «${why}» was not measured"
    return
  fi
  if said=$(python3 "scripts/$1" "${@:2}" 2>&1); then
    went_well "$why"
  else
    printf '%s\n' "$said"
    amiss "$why"
  fi
}

cd "$(dirname "$0")/.." || exit 2
unsafe_only_in_the_sys_crates
nothing_the_core_prints
nothing_the_core_depends_on
written_in_english
no_comments_in_the_code
every_example_has_its_own_name
what_python_measures "the tests live beside the file, not inside it" oversized.py --inline
what_python_measures "no file or function grows past what a person can hold" oversized.py
what_python_measures "no crate slips out of mutation" mutants_cover.py
what_python_measures "the workflows parse and name no key twice" workflows_parse.py
exit $status
