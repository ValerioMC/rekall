#!/usr/bin/env bash
# Import an H2 database from the earlier Java version of Rekall (a folder holding rekall.mv.db,
# or the file itself) into rekall.db beside it, make that folder the active database, and start
# the server on it. The H2 file is read from a copy and never changed. Reading it needs a Java
# runtime and H2's own jar: pass it with --h2-jar <path> or REKALL_H2_JAR, or let it be found in
# ~/.m2. Stop the old server first: its file is locked while it runs.
#
#   scripts/migrate-h2-to-sqlite.sh ~/rekall-data [--h2-jar h2-2.4.240.jar]
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
BIN=${REKALL_SERVER:-$ROOT/target/release/rekall-server}
[ -x "$BIN" ] || BIN=$ROOT/target/debug/rekall-server
[ -x "$BIN" ] || { echo "Build the server first: cargo build --release -p rekall-app"; exit 1; }
SOURCE=${1:?usage: $0 <folder|rekall.mv.db> [--h2-jar <jar>]}
shift
exec "$BIN" --migrate-from-h2 "$SOURCE" "$@"
