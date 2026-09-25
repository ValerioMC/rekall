# The Rust port

The backend exists twice on this branch: the Java/Spring Boot modules, unchanged, and a Rust
workspace beside them that reproduces their behaviour. `rekall-ui/` is not modified: it is the
source of truth for the wire contract, and it runs unchanged against either server.

## Layout

One crate per Maven module, in the same directory and with the same dependency direction. The Rust
sources sit next to `src/main/java` (`<module>/src/lib.rs`), so each module's two implementations
can be read side by side until the Java build is retired.

| Module | Java | Rust |
|---|---|---|
| `rekall-common` | exceptions | `RekallError`, `Id`, `Instant` (ISO_INSTANT, microseconds), `jstr` (`java.lang.String` semantics: UTF-16 lengths, `strip`, Unicode case folding), `jcoll` (a `HashSet`'s iteration order) |
| `rekall-model` | JPA entities, state rules | SeaORM entities, the same state rules and column limits |
| `rekall-repository` | Spring Data, Liquibase | SeaORM queries (one function per repository method), one migration per Liquibase changeset under the changeset's id, SQLite |
| `rekall-service` | services, `@Transactional` | services over explicit transactions (`in_write!`/`in_read!`, `BEGIN IMMEDIATE` for writes), events published after commit |
| `rekall-api` | controllers, `RestExceptionHandler`, SSE | Axum routers, the same problem bodies, the event stream |
| `rekall-mcp` | `McpController`, tools | the hand-rolled JSON-RPC endpoint, both protocol eras, the five tools with the same texts |
| `rekall-claude` | pty4j terminals, WebSocket, usage, run queue | portable-pty terminals, Axum WebSocket, usage meter, run queue runner |
| `rekall-app` | Spring Boot app, settings, backups, filter | `rekall-server`, the database registry, backups, local-access guard, SPA, actuator, in-process restart, H2 import |
| `rekall-app/desktop` | `packaging/macos/*.swift` | Tauri v2 shell running the same server in-process |

## Build, run, test

```bash
make ui                                  # the console bundle (rekall-ui/dist), embedded at compile time
cargo build --release -p rekall-app      # target/release/rekall-server
./target/release/rekall-server           # http://localhost:47355, MCP on /mcp
cargo test                               # every crate except the desktop shell
cargo build -p rekall-desktop            # needs the platform WebView SDK (WebKitGTK on Linux)
```

Properties are read as Spring Boot read them: `--name=value` arguments first, then environment
variables in relaxed form (`SERVER_PORT`, `REKALL_HOME`, `REKALL_CLAUDE_CLIPATH`, ...), then the
Java defaults. `REKALL_DB_URL` (or `spring.datasource.url`) still overrides the registry; a
`jdbc:h2:file:<base>` URL names the SQLite file `<base>.db`. `rekall.ui.dist=<folder>` serves the
console from a folder instead of the embedded bundle.

The desktop shell is a workspace member but not a default one, so `cargo build` and `cargo test`
work on a machine without the WebView SDK. `cargo tauri build` in `rekall-app/desktop` bundles it
(`Rekall.app` and a disk image on macOS). The shell was built and exercised on Linux; the macOS
bundle, the iTerm/Terminal launch and the notification prompt have not been run on a Mac yet.

## The database

H2 has no Rust driver; SQLite is the same kind of thing, one file and no server. The file is
`rekall.db`, beside where H2 kept `rekall.mv.db`, so a folder can hold both while it is migrated.
Every Liquibase changeset has a migration that does the same thing to SQLite, under the same id,
including the ones whose tables were dropped later; tables SQLite cannot alter in place are
rebuilt, with foreign keys checked after.

Moving a database from the Java build:

```bash
scripts/migrate-h2-to-sqlite.sh ~/rekall-data      # or: rekall-server --migrate-from-h2 ~/rekall-data
```

H2's own jar reads the file (from a copy; the original is never opened), the import applies exactly
the migrations whose changesets the H2 file had, copies every row, checks every reference, and runs
the rest, so a file from an older version arrives migrated as Liquibase would have done it. It then
makes that folder the active database. A registered folder that still holds only `rekall.mv.db` is
imported the first time it is opened. The import needs Java and the H2 jar (`--h2-jar`,
`REKALL_H2_JAR`, or `~/.m2`), and should run on the machine and in the time zone the Java build
ran in: H2 kept timestamps as local time.

## How parity was checked

- **The Java tests, ported.** Every JUnit class has a Rust counterpart covering its cases (a few
  service cases merged into one Rust test each): `RekallEndToEndTest` (all 104 cases), `SettingsControllerTest`,
  `BackupApiTest`, `TerminalApiTest`, `RunQueueApiTest` and the service, model, MCP and Claude
  tests. The end-to-end suites start the whole application on a real port with a file database, and
  a stub stands in for the `claude` TUI.
- **Against the Java server itself.** `scripts/parity/run.sh` starts both servers, makes the same
  ~160 calls against each (every endpoint, every MCP tool, the refusals) and diffs the answers; then
  imports the Java server's H2 file into the Rust server and reads everything back from both, so
  ids, timestamps and orders are compared on identical data. On identical data every answer
  matches in content; the differences left are listed below, and all of them are places where the
  Java server was not deterministic either.
- **The unmodified console, in a browser.** Playwright drove the built UI against `rekall-server`:
  every page, a step claimed over MCP reaching the open console over SSE, and a terminal pane
  talking to a stub `claude` over the WebSocket, with no failed request and no console error.
- **The desktop shell, under Xvfb.** The window starts the server, the folder dialog opens from the
  page, setup creates the database and the app restarts onto it, and quitting stops the server
  and closes the database.

## Where the Rust server differs, and why

Differences that remain on purpose or by necessity:

| Area | Java | Rust | Why |
|---|---|---|---|
| Database | H2 file `rekall.mv.db` | SQLite file `rekall.db` beside it | no H2 driver outside Java; see the import above |
| Backups | `BACKUP TO`, a zip holding `rekall.mv.db` | `VACUUM INTO`, a zip holding `rekall.db` | same guarantees (online, consistent); a Java backup zip is refused on restore with a pointer to the import |
| Unreadable JSON | Jackson's message (`Unexpected character ...`, `Cannot deserialize value of type ...`) | serde's message | same status (400) and shape; the parser's wording cannot be reproduced |
| Missing body | `Required request body is missing: <Java method signature>` | `Required request body is missing` | there is no Java method to name |
| Validation messages | order varied from one request to the next (Hibernate Validator's hash order) | the order the fields are declared | Java had no stable order to match |
| Context field lines | order of `Map.copyOf`, which changes with every JVM start | the order the fields are assembled in | Java had no stable order to match |
| MCP object key order | `Map.of`, which changes with every JVM start | the order written in the code | JSON object order carries no meaning; Java's was random |
| Whitelabel error page | time in the JVM's zone | time in UTC | |
| `/actuator/metrics` | JVM metrics | not served | JVM-specific; `/actuator`, `/actuator/health` (+ liveness, readiness) and `/actuator/info` are |
| Status line | `HTTP/1.1 200 ` (no reason) | `HTTP/1.1 200 OK` | not part of any contract |
| Listening socket | closed while the context restarts | kept open across an in-process restart | a request in the gap waits for the new instance instead of being refused |
| Terminal launcher (desktop) | macOS only | macOS, plus a terminal emulator on Linux | the Tauri shell also builds for Linux |
| Logs | Logback, `server.log` from the launcher's pipe | `tracing`, `server.log` written by the shell | |

Behaviour kept exactly although it looks like a bug, so it can be fixed deliberately rather than
silently:

- A path variable that is not a UUID is a **500** (`MethodArgumentTypeMismatchException`), not a 400.
- A bean-validation failure raised inside a service (not on a request body) is a **500**, not a 400.
- A column value longer than its limit is a **409** (a constraint violation), not a 400.
- Document search matches `%` and `_` in the term as `LIKE` wildcards (only `/api/documents/search`;
  `/api/search` matches them literally, as it did).
- `GET /api/backups/restore` answers 400 (it reached the download of a backup named `restore`).

Library choices that differ from the plan:

- Request validation is written out (`rekall_api::extract::Validator`) rather than with the
  `validator` crate, to produce Java's exact messages (`name: must not be blank`) and to measure
  lengths in UTF-16 code units as Bean Validation did.

## What was not ported

- The GraalVM native-image support: the reachability metadata under
  `rekall-app/src/main/resources/META-INF/native-image`, `BeanValidationRuntimeHints`,
  `LiquibaseChangeChecksumRuntimeHints`, the `aot.factories`/`spring.factories` entries,
  `@RegisterReflectionForBinding`, `NativeBindingHintsTest`, `scripts/native-build.sh` and the
  `native`/`dmg-native` Makefile targets. A Rust binary is native already.
- The Swift launcher (`packaging/macos/*.swift`, `scripts/macos-bundle.sh`,
  `scripts/macos-install.sh`), replaced by the Tauri shell; the Java build still uses them.
- `make console` (the H2 shell): the SQLite file opens with any SQLite client.
- `DatabaseLocationEnvironmentPostProcessor` as a Spring extension point: its rules live in
  `rekall_app::bootstrap::location` and run at every (re)start.
- `ApplicationRestarter`'s new Spring context: the supervisor in `rekall_app::server` builds a new
  instance over the new database in the same process.

Nothing in the Java modules was deleted; `mvn` still builds and tests them.
