# Rekall

Stores the projects and tasks you work on and hands one of them to Claude Code, in full, on a single command.

You keep projects, tasks, environments and their markdown notes in a local app; `/rk project:stvv task:code-validator` loads that task, everything it references and every note attached to any of them, without Claude ever being able to write.

## Requirements

| Tool  | Version |
|-------|---------|
| Java  | 25      |
| Maven | 3.9+    |
| Node  | 22+     |
| pnpm  | 9+      |

No database server, no Docker, no cluster. The database is an H2 file under `./data`.

## Run

```bash
make build   # builds the UI into the jar
make run     # http://localhost:8080
```

| Service | Address                     |
|---------|-----------------------------|
| UI      | `http://localhost:8080`     |
| MCP     | `http://localhost:8080/mcp` |

```bash
make reset   # delete the database file, no undo
make console # H2 shell on the database
```

## Connect Claude Code

```bash
make mcp-add          # claude mcp add --transport http rekall http://localhost:8080/mcp
make mcp-check        # verify the endpoint answers, independently of the client
cp .claude/commands/rk.md ~/.claude/commands/rk.md
```

A session then starts with one line:

```
/rk project:stvv task:code-validator-main-workflow
```

## The anchor syntax

An anchor is `entity:value`, where the entity is `project`, `task` or `environment` and the value is the name you gave the record.

| Form | Meaning |
|------|---------|
| `/rk project:stvv task:code-validator` | The project and that task, both in full |
| `/rk project:stvv` | The project, plus its tasks as a list of anchors |
| `/rk stvv code-validator` | Positional. Works while each term matches exactly one record |
| `/rk environment:"kmaster14 / stvv-dev"` | Quote a value containing spaces |

Every anchor brings back the record, what it references resolved in full **with their notes**, what references it as anchors, and its own markdown. Loading a task therefore also hands over its environment and the notes attached to that environment, which is where cluster coordinates live.

If a bare term matches more than one record the candidates come back and nothing is loaded. Task names are unique per project, so `project:` is what disambiguates a name two projects share.

One read-only tool is exposed: `rekall_context`. There is no query tool, no get tool and no schema tool. Reaching a record by asking a question in prose costs several turns before any work starts and fails silently when the guess is wrong, so the entry point is an explicit anchor and nothing else.

## Model

```
Project ──< Task >── Environment
   │          │           │
   └── notes  └── notes   └── notes
```

| Entity | Anchored by | Holds |
|--------|-------------|-------|
| `Project` | `name` | status, description, its tasks |
| `Task` | `name`, unique per project | status, description, its project, its environment |
| `Environment` | `label` | namespace, kubeconfig path |

Notes attach to exactly one of the three, enforced by a check constraint. Adding a fourth entity is a JPA class plus a Liquibase changeset, not a UI action: the schema is fixed at compile time on purpose.

## Develop

```bash
make ui-dev           # Vite dev server on :5173, proxying /api and /mcp to :8080
make test             # backend tests, then frontend lint, types and unit tests
```

### Frontend stack

| Concern | Choice |
|---|---|
| Framework | Vue 3, Composition API with `<script setup lang="ts">` only |
| Build | Vite 5 |
| Styling | Tailwind CSS 4, palette as semantic tokens in `src/assets/main.css` |
| State | Pinia setup stores |
| HTTP | `ofetch`, with timeout, retry on 5xx only and a correlation id per request |
| Validation | Zod on every response; brands are applied by the schema, so an id's kind is decided where it was checked |
| Tests | Vitest + Vue Test Utils |

```
src/
├── model/        domain types and branded ids
├── api/          http calls and Zod schemas, no state
├── stores/       Pinia
├── composables/  reusable logic without UI
├── components/
│   ├── ui/       atomic and presentational, no store, no API
│   └── shared/   composite, props in and events out
├── views/        one per route
└── router/
```

### Debug (IntelliJ IDEA)

The backend uses Lombok: Settings > Build, Execution, Deployment > Compiler > Annotation Processors > **Enable annotation processing**. Without it the IDE reports missing getters on code that compiles fine with Maven.

1. Run > Edit Configurations > Add > Remote JVM Debug, host `localhost`, port `5005`
2. Start with the debug port open:

```bash
mvn -pl rekall-app -am spring-boot:run \
  -Dspring-boot.run.jvmArguments="-agentlib:jdwp=transport=dt_socket,server=y,suspend=n,address=*:5005"
```

### Debug (VS Code)

Add to `.vscode/launch.json`:

```json
{
  "type": "java",
  "request": "attach",
  "name": "Attach to Rekall",
  "hostName": "localhost",
  "port": 5005
}
```

## Environment variables

All optional: the defaults run the application against `./data/rekall`.

| Variable             | Default | Description |
|----------------------|---------|-------------|
| `REKALL_DB_URL`      | `jdbc:h2:file:./data/rekall;AUTO_SERVER=TRUE;DB_CLOSE_DELAY=-1` | JDBC url |
| `REKALL_DB_USER`     | `rekall` | |
| `REKALL_DB_PASSWORD` | `rekall` | |
| `SERVER_PORT`        | `8080`  | |

Notes are stored in plain text in the database file. If you keep credentials in them, they are as protected as your disk is.

## Modules

```
rekall-domain/    Project, Task, Environment, Document, and the context assembly
rekall-api/       REST API for the UI
rekall-mcp/       Read-only MCP server: one tool
rekall-app/       Spring Boot entry point, serves everything
rekall-ui/        Vue 3 + Vite frontend
```

Dependencies run strictly downward. `rekall-mcp` does not depend on `rekall-api`, so no controller and no write service is on its classpath.

## Run tests

```bash
mvn test                          # everything, against an in-memory H2
cd rekall-ui && pnpm lint && pnpm typecheck && pnpm test
```

`RekallEndToEndTest` drives the real HTTP API and the real MCP endpoint: it creates a project, a task and an environment, attaches notes to two of them, and asserts that one anchored call brings back all of it. It runs against the same Liquibase changelogs the application uses, so a migration that disagrees with an entity fails there rather than at startup.

## Design

`docs/DESIGN.md` records the decisions and the reasoning behind them, including the ones that were reversed and why.
