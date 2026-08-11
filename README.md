# Rekall

Stores the structure and the working context of the projects and tasks you work on, and exposes them read-only to Claude Code over MCP.

You design entities in the UI, Rekall creates real PostgreSQL tables for them, and Claude answers questions about their contents without ever being able to write.

## Requirements

| Tool       | Version |
|------------|---------|
| Java       | 25      |
| Maven      | 3.9+    |
| Node       | 22+     |
| pnpm       | 9+      |
| Docker     | 27+     |
| minikube   | 1.35+   |
| PostgreSQL | 16+     |

## Run on minikube

```bash
# 1. Fill in the secrets
cp .env.example .env.local
$EDITOR .env.local

# 2. Cluster, database, image build, deploy, port-forwards
make up
```

| Service    | Address                 |
|------------|-------------------------|
| UI         | `http://localhost:8080` |
| MCP        | `http://localhost:8080/mcp` |
| PostgreSQL | `localhost:5432`        |

Everything runs in the minikube profile `rekall`, on its own kube context of the same name.

```bash
make stop     # pause the cluster, keep the data
make start    # resume
make down     # delete the profile and all its data
make status   # pods and services
make logs     # tail the application
make psql     # psql on the cluster database
```

## Connect Claude Code

```bash
make mcp-add          # claude mcp add --transport http rekall http://localhost:8080/mcp
make mcp-check        # verify the endpoint answers, independently of the client
```

Copy the slash command where Claude Code can see it, either in one project or for all of them:

```bash
cp .claude/commands/rk.md ~/.claude/commands/rk.md
```

A session then starts with one line:

```
/rk project:stvv task:code-validator-main-workflow
```

## The anchor syntax

An anchor is `entity:value`. The entity part matches an entity's physical name, its label, its plural label or any alias, case-insensitively. The value matches that entity's display field.

| Form | Meaning |
|------|---------|
| `/rk project:stvv task:code-validator` | Two anchors. Both records load in full |
| `/rk project:stvv` | One anchor. The project, plus its tasks as a list of anchors |
| `/rk stvv code-validator` | Positional. Works while each term matches exactly one record anywhere |
| `/rk environment:"kmaster14 / stvv-dev"` | Quote a value containing spaces |

Every anchor brings back the record, the records it references resolved in full, the records referencing it as labels, and all of its markdown documents. If a term matches more than one record the candidates come back and nothing is loaded: the tool never picks for you.

Two read-only tools are exposed.

| Tool             | Purpose |
|------------------|---------|
| `rekall_context` | Load the records named by a list of anchors, with everything attached to them |
| `rekall_schema`  | Every entity, its fields, relations and aliases. This is how you find out which anchors exist |

There is no query tool, no get tool and no search tool. Reaching a record by asking a question in prose costs several turns before any work starts and fails silently when the guess is wrong, so the entry point is an explicit anchor and nothing else.

## Develop

```bash
make run-local        # backend against a port-forwarded database
make ui-dev           # Vite dev server on :5173, proxying /api and /mcp to :8080
make test             # backend tests + frontend lint, types and unit tests
```

### Frontend stack

| Concern | Choice |
|---|---|
| Framework | Vue 3, Composition API with `<script setup lang="ts">` only |
| Build | Vite 5 |
| Styling | Tailwind CSS 4, with the palette defined as semantic tokens in `src/assets/main.css` |
| State | Pinia setup stores |
| HTTP | `ofetch`, with timeout, retry on 5xx only and a correlation id per request |
| Validation | Zod on every response; brands are applied by the schema, so an id's kind is decided where it was checked |
| Tests | Vitest + Vue Test Utils, Playwright for the screens |
| Tooling | ESLint + Prettier, pnpm |

```
src/
├── model/        domain types, branded ids, pure mappers
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

The backend uses Lombok, so before anything else: Settings > Build, Execution, Deployment > Compiler > Annotation Processors > **Enable annotation processing**. Without it the IDE reports missing getters and constructors on code that compiles fine with Maven.

1. Run > Edit Configurations > Add > Remote JVM Debug, host `localhost`, port `5005`
2. Start with the debug port open:

```bash
set -a; . ./.env.local; set +a
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

| Variable                 | Required | Default | Description |
|--------------------------|----------|---------|-------------|
| `REKALL_READER_PASSWORD` | yes      | none    | Password of the read-only role the MCP server uses. Liquibase creates the role with it. No default exists anywhere: an unresolved value stops the application on purpose |
| `REKALL_DB_URL`          | no       | `jdbc:postgresql://localhost:5432/rekall` | JDBC url |
| `REKALL_DB_USER`         | no       | `rekall` | Application role, holds DDL and DML |
| `REKALL_DB_PASSWORD`     | no       | `rekall` | |
| `SERVER_PORT`            | no       | `8080`  | |

## Modules

```
rekall-meta/      Meta-model entities, Liquibase changelogs, identifier validation
rekall-engine/    jOOQ DDL planner and executor, dynamic record access
rekall-content/   Markdown documents, full-text search, folder import and export
rekall-api/       REST API for the UI
rekall-mcp/       Read-only MCP server
rekall-app/       Spring Boot entry point, serves everything
rekall-ui/        Vue 3 + Vite frontend
```

Dependencies run strictly downward. `rekall-mcp` does not depend on `rekall-api`, so no write path is on its classpath.

## How the schema works

The database has two schemas with two different owners.

| Schema        | Owner              | Contents |
|---------------|--------------------|----------|
| `rekall_meta` | Liquibase          | The meta-model, the DDL log, the documents |
| `rekall_data` | The jOOQ DDL engine | The tables you define. Liquibase creates the schema and never authors anything inside it |

Editing an entity changes only its definition. `GET /api/meta/plan` computes the difference against the live schema and classifies every change:

| Class         | Meaning |
|---------------|---------|
| `SAFE`        | Cannot lose data. Applied without asking |
| `NEEDS_INPUT` | Needs a default for existing rows, or a confirmation to destroy something |
| `BLOCKED`     | Refused. A narrowing varchar or an unrelated type change would need a cast whose failure mode is silent data loss |

Apply runs the whole plan inside one transaction. PostgreSQL has transactional DDL, so a statement failing halfway leaves no partially migrated schema. That is why PostgreSQL is a requirement rather than a preference.

Physical table and column names are immutable once defined; labels are freely editable. A diff cannot tell a rename from a drop plus an add, and guessing wrong would silently destroy a column.

## Run tests

```bash
mvn test                          # everything, against real PostgreSQL via Testcontainers
mvn -pl rekall-engine test        # the DDL engine, one test per alter rule
mvn -pl rekall-app test           # end to end, plus every endpoint the UI calls

cd rekall-ui
pnpm lint                         # eslint, zero warnings tolerated
pnpm typecheck                    # vue-tsc, strict with noUncheckedIndexedAccess
pnpm test                         # vitest

make test-ui-e2e                  # drives the deployed UI in a headless browser
```

The engine is tested against a real PostgreSQL container rather than an in-memory database: generated DDL, `information_schema` reflection and transactional DDL are all Postgres-specific behaviour, and an in-memory substitute would test none of them.

`UiEndpointCoverageTest` walks every endpoint the frontend calls, in the order it calls them. It exists because the API maps entities to DTOs after the service transaction closes, so an association left lazy fails at serialisation time and nowhere else.

`make test-ui-e2e` needs a running instance and Playwright, and is therefore not part of `make test`. It catches what the API tests structurally cannot: a screen that renders but calls the wrong endpoint, or a component that throws after mount. It fails on any console error, any failed request and any 4xx or 5xx response, and it cleans up the throwaway entity it creates so it can be run repeatedly.

## Design

`docs/DESIGN.md` records the decisions and the reasoning behind them, including the ones that were deliberately rejected.
