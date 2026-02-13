#!/usr/bin/env bash
set -euo pipefail

interval_seconds="${GROVE_FAKE_CODEX_INTERVAL_SEC:-0.02}"

printf 'fake codex boot\n'

index=0
while true; do
  printf '\033[38;5;39mthinking frame %04d\033[0m\n' "${index}"
  printf '[<35;192;47M'
  printf '\033[?1000h'
  printf '\033[?1000l'
  if (( index % 7 == 0 )); then
    printf 'allow edit? [y/n]\n'
  fi
  sleep "${interval_seconds}"
  index=$((index + 1))
done
