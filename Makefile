# Rekall runs as one process against an H2 file. There is no cluster, no container and no
# database server to start, so this file is short on purpose.

MVN         ?= mvn
PNPM        ?= pnpm
JAR         := rekall-app/target/rekall-app-0.1.0-SNAPSHOT.jar
NATIVE_BIN  := rekall-app/target/rekall-app
UI          := rekall-ui
DB          := ./data/rekall.mv.db
REKALL_URL  ?= http://localhost:8080

# A database of its own, never the one `run`/`reset`/`console` point at (which, once a
# database folder has been chosen through the setup wizard, is not `./data` at all — see
# DatabaseLocationEnvironmentPostProcessor). `load-data` writes real-looking demo companies
# by the dozen; it must never be able to land in the database you actually use.
DEMO_DB_DIR             := ./data/demo
DEMO_DB_URL             := jdbc:h2:file:$(CURDIR)/$(DEMO_DB_DIR)/rekall;AUTO_SERVER=TRUE;DB_CLOSE_DELAY=-1
DEMO_COMPANIES          ?= 3
DEMO_TASKS_PER_COMPANY  ?= 20

.DEFAULT_GOAL := help
.PHONY: help run run-demo build jar native run-native start-native dmg-native dmg-jvm ui ui-dev test test-backend test-ui mcp-add mcp-check console reset reset-data load-data

help: ## Show this help
	@grep -hE '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk -F':.*?## ' '{printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'

# The compiled frontend is committed under rekall-app/src/main/resources/static, so nothing
# fails when it is out of date: Spring serves the stale bundle and the browser shows a version
# of the application that no longer exists. Every target that serves the UI rebuilds it first.
ui: ## Compile the frontend into rekall-app/src/main/resources/static
	cd $(UI) && $(PNPM) install --frozen-lockfile && $(PNPM) build

# workingDirectory is pinned to the repository root because spring-boot:run otherwise runs from
# the module directory, which makes the `./data` in the datasource url resolve to
# rekall-app/data. Running the packaged jar from here resolves it to ./data, so the application
# had two databases depending on how it was started, and `reset` and `console` addressed the one
# that was not in use.
run: ui ## Start the application on http://localhost:8080
	$(MVN) -pl rekall-app -am spring-boot:run -Dspring-boot.run.workingDirectory=$(CURDIR)

run-demo: ui ## Start the app on a throwaway database at ./data/demo, for load-data — your real database is untouched
	REKALL_DB_URL="$(DEMO_DB_URL)" $(MVN) -pl rekall-app -am spring-boot:run -Dspring-boot.run.workingDirectory=$(CURDIR)

build: ui ## Build the UI into the jar and package it
	$(MVN) -q clean package

jar: build ## Alias for build
	@echo "$(JAR)"

# See scripts/native-build.sh for why this isn't just `mvn -Pnative package`: the
# ByteBuddy/Hibernate fix needs a jar patched outside anything Maven can do reliably here.
# It never touches ~/.m2 with the patched jar - that copy lives only under
# rekall-app/target/native-libs, used solely for the native-image classpath.
native: ui ## Build the GraalVM native binary (needs GraalVM as JAVA_HOME, takes ~5-8 minutes)
	./scripts/native-build.sh

run-native: native ## Build and start the GraalVM native binary on http://localhost:8080
	./$(NATIVE_BIN)

start-native: ## Start the already-built native binary, no rebuild - fails if `make native` hasn't run yet
	@test -x $(NATIVE_BIN) || { echo "$(NATIVE_BIN) not found - run 'make native' first"; exit 1; }
	./$(NATIVE_BIN)

# macOS only, and additive: everything above stays the plain build that has to keep working on
# Windows and Linux. These two wrap it in Rekall.app - a window that starts the server itself -
# and hand back a disk image to drag into /Applications. See packaging/macos/Launcher.swift.
dmg-native: native ## macOS: package the native binary as Rekall.app in a DMG (needs GraalVM as JAVA_HOME)
	./scripts/macos-bundle.sh native

dmg-jvm: build ## macOS: package the jar plus a bundled Java runtime as Rekall.app in a DMG
	./scripts/macos-bundle.sh jvm

ui-dev: ## Vite dev server on :5173, proxying /api and /mcp to :8080
	cd $(UI) && $(PNPM) dev

test: test-backend test-ui ## Everything

test-backend: ## JUnit against an in-memory H2
	$(MVN) test

test-ui: ## eslint, vue-tsc and vitest
	cd $(UI) && $(PNPM) lint && $(PNPM) typecheck && $(PNPM) test

mcp-add: ## Register the MCP server with Claude Code
	claude mcp add --transport http rekall http://localhost:8080/mcp

mcp-check: ## Verify the endpoint answers, independently of the client
	@curl -sf -X POST http://localhost:8080/mcp \
		-H 'Content-Type: application/json' \
		-d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
		| grep -q rekall_context && echo "mcp ok" || (echo "mcp not answering"; exit 1)

console: ## Open an H2 shell on the database file
	@java -cp $$(find ~/.m2/repository/com/h2database/h2 -name 'h2-*.jar' | head -1) \
		org.h2.tools.Shell -url "jdbc:h2:file:./data/rekall" -user rekall -password rekall

reset: ## Delete the database file. There is no undo
	rm -f $(DB) ./data/rekall.trace.db
	@echo "database deleted"

reset-data: ## Delete the demo database (./data/demo) so the next load-data starts clean. Never touches `run`'s database
	rm -rf $(DEMO_DB_DIR)
	@echo "demo database deleted"

load-data: ## Seed the running instance with demo companies, tasks, notes and time tracking for the current month. Start it first with `make run-demo`
	python3 scripts/seed-demo-data.py --base-url $(REKALL_URL) --companies $(DEMO_COMPANIES) --tasks-per-company $(DEMO_TASKS_PER_COMPANY)
