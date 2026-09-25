# Rekall runs as one process against a SQLite file. There is no cluster, no container and no
# database server to start, so this file is short on purpose.

PNPM        ?= pnpm
CARGO       ?= cargo
UI          := rekall-ui
SERVER_BIN  := target/release/rekall-server
DESKTOP     := rekall-app/desktop
REKALL_URL  ?= http://localhost:47355

# A database of its own, never the one `run` points at (which is whatever folder the setup
# wizard recorded in ~/.rekall/config.json). `load-data` writes real-looking demo companies by
# the dozen; it must never be able to land in the database you actually use.
DEMO_DB_DIR             := ./data/demo
DEMO_DB_URL             := sqlite:$(CURDIR)/$(DEMO_DB_DIR)/rekall.db
DEMO_COMPANIES          ?= 3
DEMO_TASKS_PER_COMPANY  ?= 20

.DEFAULT_GOAL := help
.PHONY: help ui ui-dev run run-demo build start desktop dmg app icons test test-backend test-ui lint \
	mcp-add mcp-check reset-data load-data import-h2

help: ## Show this help
	@grep -hE '^[a-zA-Z0-9_-]+:.*?## ' $(MAKEFILE_LIST) | awk -F':.*?## ' '{printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'

# The bundle is build output: vite writes it to rekall-ui/dist, git ignores it, and rekall-app
# embeds it into the binary at compile time. So it can be absent or stale, and every target that
# serves the UI rebuilds it first rather than trusting whatever was left behind.
ui: ## Compile the frontend into rekall-ui/dist, which the server embeds
	cd $(UI) && $(PNPM) install --frozen-lockfile && $(PNPM) build

ui-dev: ## Vite dev server on :5173, proxying /api and /mcp to :47355
	cd $(UI) && $(PNPM) dev

run: ui ## Start the server on http://localhost:47355
	$(CARGO) run --release -p rekall-app --bin rekall-server

run-demo: ui ## Start the server on a throwaway database at ./data/demo, for load-data. Your real database is untouched
	mkdir -p $(DEMO_DB_DIR)
	REKALL_DB_URL="$(DEMO_DB_URL)" $(CARGO) run --release -p rekall-app --bin rekall-server

build: ui ## Build rekall-server, console embedded, into target/release
	$(CARGO) build --release -p rekall-app
	@echo "$(SERVER_BIN)"

start: ## Start the already-built server, no rebuild. Fails if `make build` hasn't run yet
	@test -x $(SERVER_BIN) || { echo "$(SERVER_BIN) not found - run 'make build' first"; exit 1; }
	./$(SERVER_BIN)

desktop: ui ## Build the desktop app binary (Tauri; needs the platform WebView SDK)
	$(CARGO) build --release -p rekall-desktop

# macOS: Rekall.app and the disk image to hand to another machine, both under
# target/release/bundle. Needs the Tauri CLI: cargo install tauri-cli --version "^2" --locked
dmg: ui ## macOS: bundle the desktop app as Rekall.app and a disk image under target/release/bundle
	@$(CARGO) tauri --version >/dev/null 2>&1 || { echo "The Tauri CLI is missing: cargo install tauri-cli --version '^2' --locked"; exit 1; }
	cd $(DESKTOP) && $(CARGO) tauri build --bundles app,dmg

# Nothing is copied into /Applications: the disk image is mounted and Rekall runs from it, so
# what you try is exactly what another machine gets, and the installed copy (if any) is left alone.
app: dmg ## macOS: build the disk image, mount it and run Rekall from it, without installing it
	./scripts/macos-app.sh

icons: ## macOS: re-render the PWA, touch and desktop icons from rekall-ui/public/favicon.svg (needs Google Chrome)
	./scripts/render-icons.sh

test: test-backend test-ui ## Everything

test-backend: ## Every crate's tests, except the desktop shell's
	$(CARGO) test

test-ui: ## eslint, vue-tsc and vitest
	cd $(UI) && $(PNPM) lint && $(PNPM) typecheck && $(PNPM) test

lint: ## clippy over every crate and test, as CI runs it
	$(CARGO) clippy --all-targets

# --scope user, not the default local scope: a local registration exists only for the one
# directory it was run from, which looks identical from inside that directory and like nothing
# at all from anywhere else. Settings > Claude Code does the same thing from the running
# application, and installs the /rk command with it.
mcp-add: ## Register the MCP server with Claude Code, for every folder
	claude mcp add --scope user --transport http rekall http://localhost:47355/mcp

mcp-check: ## Verify the endpoint answers, independently of the client
	@curl -sf -X POST http://localhost:47355/mcp \
		-H 'Content-Type: application/json' \
		-d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
		| grep -q rekall_context && echo "mcp ok" || (echo "mcp not answering"; exit 1)

reset-data: ## Delete the demo database (./data/demo) so the next load-data starts clean. Never touches `run`'s database
	rm -rf $(DEMO_DB_DIR)
	@echo "demo database deleted"

load-data: ## Seed the running instance with demo companies, tasks, notes and time tracking. Start it first with `make run-demo`
	python3 scripts/seed-demo-data.py --base-url $(REKALL_URL) --companies $(DEMO_COMPANIES) --tasks-per-company $(DEMO_TASKS_PER_COMPANY)

import-h2: build ## Import a legacy H2 database (DIR holding rekall.mv.db) into SQLite and make it the active one
	@test -n "$(DIR)" || { echo "usage: make import-h2 DIR=<folder holding rekall.mv.db>"; exit 1; }
	./scripts/migrate-h2-to-sqlite.sh "$(DIR)"
