#!/usr/bin/env bash
# Behavioural parity between the Java server and the Rust one.
#
#   1. Both start from nothing on ports 47401 (Java) and 47402 (Rust); calls.py makes the same
#      ~170 calls against each (every endpoint, the MCP tools, the refusals) and diff.py compares
#      the answers with ids, timestamps and per-run folders normalised.
#   2. The Java server's H2 file is then imported into SQLite with `rekall-server
#      --migrate-from-h2`; reads.py reads everything back from the Java server and from the Rust
#      one on the imported copy, so ids, timestamps and orders can be compared as they are.
#
# Needs the Java jar (`mvn -DskipTests package`), the Rust binary (`cargo build -p rekall-app`),
# git, python3 and the H2 jar in the local Maven repository (or REKALL_H2_JAR).
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
WORK=${WORK:-$(mktemp -d)}
JAR=$ROOT/rekall-app/target/rekall-app-0.1.0-SNAPSHOT.jar
BIN=$ROOT/target/debug/rekall-server
HERE=$ROOT/scripts/parity
echo "Working in $WORK"
mkdir -p "$WORK/java/db" "$WORK/java/home" "$WORK/rust/db" "$WORK/rust/home" "$WORK/imported/home" "$WORK/plain" "$WORK/repo"
git -C "$WORK/repo" init -q -b main
git -C "$WORK/repo" config user.name Test && git -C "$WORK/repo" config user.email test@example.com
git -C "$WORK/repo" config commit.gpgsign false
echo one > "$WORK/repo/README.md" && git -C "$WORK/repo" add . && git -C "$WORK/repo" commit -qm "Add the readme"
echo two >> "$WORK/repo/README.md" && git -C "$WORK/repo" commit -qam "Wire up the ledger"

pids=()
cleanup() { for pid in "${pids[@]}"; do kill "$pid" 2>/dev/null || true; done; }
trap cleanup EXIT
wait_for() { for _ in $(seq 1 120); do curl -sf "localhost:$1/actuator/health" >/dev/null && return; sleep 1; done; echo "port $1 never came up"; exit 1; }
start_java() {
  REKALL_DB_URL="jdbc:h2:file:$WORK/java/db/rekall;DB_CLOSE_DELAY=-1" java -Drekall.home="$WORK/java/home/.rekall" \
    -Duser.home="$WORK/java/home" -jar "$JAR" --server.port=47401 --rekall.backup.enabled=false > "$WORK/java.log" 2>&1 &
  pids+=($!); wait_for 47401
}

start_java
REKALL_DB_URL="sqlite:$WORK/rust/db/rekall.db" "$BIN" --server.port=47402 --rekall.home="$WORK/rust/home/.rekall" \
  --user.home="$WORK/rust/home" --rekall.backup.enabled=false > "$WORK/rust.log" 2>&1 &
pids+=($!); wait_for 47402

python3 "$HERE/calls.py" http://localhost:47401 "$WORK/repo" "$WORK/plain" "$WORK/calls-java.json" "$WORK/java" --keep
python3 "$HERE/calls.py" http://localhost:47402 "$WORK/repo" "$WORK/plain" "$WORK/calls-rust.json" "$WORK/rust" --keep
python3 "$HERE/diff.py" "$WORK/calls-java.json" "$WORK/calls-rust.json" | tee "$WORK/calls.diff" | tail -1

cleanup; pids=(); sleep 3
"$BIN" --migrate-from-h2 "$WORK/java/db" --server.port=47403 --rekall.home="$WORK/imported/home/.rekall" \
  --user.home="$WORK/imported/home" --rekall.backup.enabled=false > "$WORK/imported.log" 2>&1 &
pids+=($!)
start_java; wait_for 47403
python3 "$HERE/reads.py" http://localhost:47401 "$WORK/reads-java.json"
python3 "$HERE/reads.py" http://localhost:47403 "$WORK/reads-rust.json"
python3 "$HERE/diff.py" "$WORK/reads-java.json" "$WORK/reads-rust.json" | tee "$WORK/reads.diff" | tail -1
