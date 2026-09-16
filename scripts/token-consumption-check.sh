#!/bin/sh
# Compares token usage between the flag set the in-app "Run here" path uses
# (ClaudeProcessManager) and the plain invocation the "Open in Claude Code"
# native launcher uses (ClaudeCodeLauncher). Same prompt, same directory,
# same environment. Reads the `usage` block off the final `result` event.
#
# Usage: scripts/token-consumption-check.sh "a fixed prompt"

set -eu

PROMPT="${1:-Respond with the single word: pong. Do not use any tools.}"
CLAUDE="$(command -v claude)"
WORKDIR="$(pwd)"

echo "claude:   $CLAUDE ($($CLAUDE --version))"
echo "workdir:  $WORKDIR"
echo "prompt:   $PROMPT"
echo

usage_from_stream() {
  # last line of stream-json is the result event
  grep '"type":"result"' | tail -1 \
    | python3 -c 'import sys,json; d=json.load(sys.stdin); u=d.get("usage",{}); print(json.dumps({k:u.get(k) for k in ["input_tokens","cache_creation_input_tokens","cache_read_input_tokens","output_tokens"]}), "cost_usd=", d.get("total_cost_usd"), "model=", d.get("modelUsage") and list(d["modelUsage"].keys()))'
}

usage_from_json() {
  python3 -c 'import sys,json; d=json.load(sys.stdin); u=d.get("usage",{}); print(json.dumps({k:u.get(k) for k in ["input_tokens","cache_creation_input_tokens","cache_read_input_tokens","output_tokens"]}), "cost_usd=", d.get("total_cost_usd"), "model=", d.get("modelUsage") and list(d["modelUsage"].keys()))'
}

stream_line() {
  python3 -c 'import json,sys; print(json.dumps({"type":"user","message":{"role":"user","content":[{"type":"text","text":sys.argv[1]}]}}))' "$1"
}

echo "== A: in-app 'Run here' flags (stream-json in/out, --verbose), prompt on stdin =="
stream_line "$PROMPT" | "$CLAUDE" --print --verbose \
  --input-format stream-json --output-format stream-json \
  | usage_from_stream
echo

echo "== B: 'Open in Claude Code' plain invocation (measured via --output-format json) =="
"$CLAUDE" --print --output-format json "$PROMPT" | usage_from_json
echo

echo "== C: in-app flags + explicit --model sonnet =="
stream_line "$PROMPT" | "$CLAUDE" --print --verbose --model sonnet \
  --input-format stream-json --output-format stream-json \
  | usage_from_stream
