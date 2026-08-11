PROFILE    := rekall
NAMESPACE  := rekall
KUBECTL    := kubectl -n $(NAMESPACE)
PID_FILE   := .forward.pids
ENV_FILE   := .env.local

APP_URL    := http://localhost:8080
MCP_URL    := $(APP_URL)/mcp

.DEFAULT_GOAL := help

.PHONY: help up down start stop build deploy port-forward stop-forward secrets status \
        logs psql mcp-add mcp-check test test-backend test-ui test-ui-e2e ui-dev run-local k9s \
        _wait-infra _wait-app _require-env

# ---------------------------------------------------------------
# Public targets
# ---------------------------------------------------------------

help:
	@echo ""
	@echo "rekall"
	@echo ""
	@echo "  make up            full setup: minikube + postgres + build + deploy + port-forward"
	@echo "  make down          delete the minikube profile (destroys all data)"
	@echo "  make start         start the existing cluster + port-forwards"
	@echo "  make stop          pause the cluster, kill port-forwards (preserves data)"
	@echo ""
	@echo "  make build         rebuild the rekall image inside minikube"
	@echo "  make deploy        re-apply manifests and restart the deployment"
	@echo "  make port-forward  start background port-forwards"
	@echo "  make stop-forward  kill background port-forwards"
	@echo "  make status        show pod and service status"
	@echo "  make logs          tail the rekall logs"
	@echo "  make psql          open psql on the cluster database"
	@echo ""
	@echo "  make mcp-add       register the MCP server with Claude Code"
	@echo "  make mcp-check     call the MCP endpoint and list its tools"
	@echo ""
	@echo "  make test          run every test (backend + frontend lint, types, unit)"
	@echo "  make test-ui-e2e   drive the deployed UI in a headless browser"
	@echo "  make run-local     run the backend against a local postgres, no cluster"
	@echo "  make ui-dev        run the Vite dev server against localhost:8080"
	@echo ""

# Full one-shot setup
up: _require-env
	@echo "==> Starting minikube (profile: $(PROFILE))"
	minikube start \
	  --profile $(PROFILE) \
	  --driver docker \
	  --cpus 4 \
	  --memory 6144 \
	  --disk-size 20g || true
	kubectl config use-context $(PROFILE)

	@echo "==> Applying namespace"
	kubectl apply -f k8s/namespace.yaml

	@$(MAKE) secrets

	@echo "==> Deploying postgres"
	$(KUBECTL) apply -f k8s/postgres/
	@$(MAKE) _wait-infra

	@echo "==> Building the rekall image inside minikube"
	@$(MAKE) build

	@echo "==> Deploying rekall"
	$(KUBECTL) apply -f k8s/rekall/
	@$(MAKE) _wait-app

	@$(MAKE) port-forward
	@echo ""
	@echo "==> Done."
	@echo "    UI          $(APP_URL)"
	@echo "    MCP         $(MCP_URL)"
	@echo "    PostgreSQL  localhost:5432"
	@echo ""
	@echo "    Run 'make mcp-add' to register the MCP server with Claude Code."
	@echo ""

start:
	minikube start --profile $(PROFILE)
	kubectl config use-context $(PROFILE)
	@$(MAKE) port-forward

stop: stop-forward
	minikube stop --profile $(PROFILE)

down: stop-forward
	minikube delete --profile $(PROFILE)
	@rm -f $(PID_FILE)

# ---------------------------------------------------------------
# Secrets
# ---------------------------------------------------------------

# Read from .env.local, which is gitignored. Nothing here has a default: a well-known
# fallback password on a role that can read every note is worse than a failed make.
_require-env:
	@test -f $(ENV_FILE) || { \
	  echo "Missing $(ENV_FILE). Copy .env.example and fill it in:"; \
	  echo "  cp .env.example $(ENV_FILE)"; \
	  exit 1; }

secrets: _require-env
	@echo "==> Applying secrets"
	@set -a; . ./$(ENV_FILE); set +a; \
	kubectl create secret generic rekall-secrets -n $(NAMESPACE) \
	  --from-literal=db-user="$$REKALL_DB_USER" \
	  --from-literal=db-password="$$REKALL_DB_PASSWORD" \
	  --from-literal=reader-password="$$REKALL_READER_PASSWORD" \
	  --dry-run=client -o yaml | kubectl apply -f -

# ---------------------------------------------------------------
# Build & deploy
# ---------------------------------------------------------------

build:
	@echo "==> Building rekall:latest inside the minikube docker daemon"
	eval $$(minikube docker-env --profile $(PROFILE)) && \
	docker build -t rekall:latest .

deploy:
	$(KUBECTL) apply -f k8s/rekall/
	$(KUBECTL) rollout restart deployment/rekall
	$(KUBECTL) rollout status deployment/rekall --timeout=300s

# ---------------------------------------------------------------
# Port-forward management
# ---------------------------------------------------------------

port-forward: stop-forward
	@echo "==> Starting port-forwards (background)"
	@kubectl port-forward svc/rekall   8080:8080 -n $(NAMESPACE) > /dev/null 2>&1 & echo $$! >> $(PID_FILE)
	@kubectl port-forward svc/postgres 5432:5432 -n $(NAMESPACE) > /dev/null 2>&1 & echo $$! >> $(PID_FILE)
	@sleep 2
	@echo ""
	@echo "  UI          $(APP_URL)"
	@echo "  MCP         $(MCP_URL)"
	@echo "  PostgreSQL  localhost:5432   db: rekall"
	@echo ""

stop-forward:
	@if [ -f $(PID_FILE) ]; then \
	  echo "==> Stopping port-forwards"; \
	  xargs kill 2>/dev/null < $(PID_FILE) || true; \
	  rm -f $(PID_FILE); \
	fi

# ---------------------------------------------------------------
# Claude Code integration
# ---------------------------------------------------------------

mcp-add:
	claude mcp add --transport http rekall $(MCP_URL)
	@echo "Registered. Ask Claude Code: 'Che progetti abbiamo attivi?'"

# Useful on its own: it proves the endpoint answers before blaming the client.
mcp-check:
	@curl -s -X POST $(MCP_URL) \
	  -H 'Content-Type: application/json' \
	  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
	  | python3 -m json.tool

# ---------------------------------------------------------------
# Observability
# ---------------------------------------------------------------

status:
	@echo "--- Pods ---"
	$(KUBECTL) get pods
	@echo ""
	@echo "--- Services ---"
	$(KUBECTL) get svc

logs:
	$(KUBECTL) logs -f deployment/rekall

psql: _require-env
	@set -a; . ./$(ENV_FILE); set +a; \
	$(KUBECTL) exec -it deployment/postgres -- psql -U "$$REKALL_DB_USER" -d rekall

k9s:
	k9s --context $(PROFILE) --namespace $(NAMESPACE)

# ---------------------------------------------------------------
# Development
# ---------------------------------------------------------------

test: test-backend test-ui

test-backend:
	mvn -B test

test-ui:
	cd rekall-ui && pnpm lint && pnpm typecheck && pnpm test

# Drives the deployed UI in a headless browser. Needs a running instance, so it is not part
# of `make test`: it verifies the screens, which the API tests structurally cannot.
test-ui-e2e:
	python3 rekall-ui/e2e/ui-smoke.py

ui-dev:
	cd rekall-ui && pnpm dev

# Runs against whatever postgres is on localhost:5432, which is the cluster one when
# port-forwards are up.
run-local: _require-env
	set -a; . ./$(ENV_FILE); set +a; \
	mvn -B -pl rekall-app -am spring-boot:run

# ---------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------

_wait-infra:
	@echo "==> Waiting for postgres"
	$(KUBECTL) rollout status deployment/postgres --timeout=180s

_wait-app:
	@echo "==> Waiting for rekall"
	$(KUBECTL) rollout status deployment/rekall --timeout=300s
