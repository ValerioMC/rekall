# Rekall runs as one process against an H2 file. There is no cluster, no container and no
# database server to start, so this file is short on purpose.

MVN     ?= mvn
PNPM    ?= pnpm
JAR     := rekall-app/target/rekall-app-0.1.0-SNAPSHOT.jar
UI      := rekall-ui
DB      := ./data/rekall.mv.db

.DEFAULT_GOAL := help
.PHONY: help run build jar ui-dev test test-backend test-ui mcp-add mcp-check console reset

help: ## Show this help
	@grep -hE '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk -F':.*?## ' '{printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'

run: ## Start the application on http://localhost:8080
	$(MVN) -pl rekall-app -am spring-boot:run

build: ## Build the UI into the jar and package it
	cd $(UI) && $(PNPM) install --frozen-lockfile && $(PNPM) build
	$(MVN) -q clean package

jar: build ## Alias for build
	@echo "$(JAR)"

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
