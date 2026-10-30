#!/bin/sh
exec 3< "$FAKE_CLAUDE_FIXTURE"

emit_turn() {
  while IFS= read -r line <&3; do
    printf '%s\n' "$line"
    case "$line" in
      *'"type":"result"'*) return 0 ;;
    esac
  done
}

while IFS= read -r input; do
  printf '%s\n' "$input" >> "$FAKE_CLAUDE_RECEIVED"
  case "$input" in
    *'"subtype":"interrupt"'*) continue ;;
  esac
  emit_turn
  if [ -n "$FAKE_CLAUDE_CUT_MID_LINE" ]; then
    printf '{"type":"assis'
    exit 0
  fi
  if [ -n "$FAKE_CLAUDE_EXIT_CODE" ]; then
    exit "$FAKE_CLAUDE_EXIT_CODE"
  fi
done
