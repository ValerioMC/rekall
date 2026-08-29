#!/usr/bin/env python3
"""Populates a running Rekall instance with demo companies, projects, tasks, notes and
time tracking for the current month.

Goes entirely through the public REST API, the same one the console uses, so the data it
produces looks exactly like data a person made by hand: sessions are opened and closed
(and, for one task per company, left open) through the timer endpoints and then corrected
by hand to land on a plausible day and hour this month, the same path `TimeLogDialog`
gives you.

Meant to run against a throwaway database (see `make run-demo` / `make reset-data` in the
Makefile) — it refuses to run against an instance that already has data unless told to
with --force, so it cannot dump demo clutter into a real, populated console by accident.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import sys
import urllib.error
import urllib.request
from datetime import datetime, timedelta

COMPANIES = [
    ("Acme Robotics", "Industrial automation and warehouse robotics.",
     ["chassis", "navigation", "vision-pipeline", "fleet-ops"]),
    ("Globex Analytics", "A data platform for retail demand forecasting.",
     ["ingestion", "warehouse", "dashboards", "ml-models"]),
    ("Initech Labs", "Developer tooling and internal platform engineering.",
     ["ci-pipeline", "auth-service", "design-system", "observability"]),
    ("Umbrella Systems", "Healthcare records and compliance software.",
     ["patient-portal", "billing", "compliance", "integrations"]),
    ("Soylent Digital", "A consumer nutrition and subscription app.",
     ["mobile-app", "checkout", "recommendations", "growth"]),
    ("Stark Dynamics", "Energy management and IoT devices.",
     ["firmware", "cloud-sync", "energy-dashboard", "device-provisioning"]),
]

TASK_TITLES = [
    "Set up CI pipeline", "Fix login redirect bug", "Write onboarding docs",
    "Refactor auth module", "Add rate limiting", "Investigate memory leak",
    "Migrate database schema", "Improve error messages", "Add dark mode toggle",
    "Write integration tests", "Optimize query performance", "Update dependencies",
    "Design API contract", "Implement webhook retries", "Add pagination to list endpoint",
    "Fix flaky test", "Set up monitoring dashboard", "Draft RFC for caching layer",
    "Clean up dead code", "Add feature flag support", "Improve build times",
    "Write incident postmortem", "Add CSV export", "Implement search autocomplete",
    "Fix timezone bug", "Add rate-limit headers", "Upgrade framework version",
    "Harden input validation", "Add audit logging", "Simplify config loading",
    "Reduce bundle size", "Add retry with backoff", "Fix N+1 query",
    "Write load test scenarios", "Add health check endpoint", "Deprecate legacy endpoint",
    "Add keyboard shortcuts", "Improve empty states", "Fix race condition on save",
    "Add structured logging", "Rotate API credentials", "Write ADR for message queue",
    "Add optimistic UI updates", "Fix off-by-one in pagination", "Add feature usage metrics",
    "Consolidate duplicate components", "Add request tracing", "Fix broken CSV import",
]

TASK_STATUSES = ["TODO", "IN_PROGRESS", "BLOCKED", "DONE"]
TASK_STATUS_WEIGHTS = [30, 35, 10, 25]

DOCUMENT_KINDS = ["notes", "notes", "notes", "context", "architecture"]

NOTE_OPENERS = [
    "Current implementation reads straightforwardly; the tricky part is the edge cases.",
    "Talked this through with the team — approach below is what we settled on.",
    "Still exploratory. Leaving the options open below rather than picking one too early.",
    "Mostly done, a couple of loose ends called out below.",
]
NOTE_BULLETS = [
    "Needs a second pass once the API contract is finalised.",
    "Watch out for the timezone handling here.",
    "Covered by the integration suite, not yet by anything faster.",
    "Left a TODO in the code pointing back here.",
    "Depends on the migration landing first.",
    "No regressions seen locally; worth a canary before wider rollout.",
]

WORKDAY_START_HOUR = 8
WORKDAY_END_HOUR = 18


def slugify(text: str) -> str:
    keep = "".join(c.lower() if c.isalnum() else "-" for c in text)
    while "--" in keep:
        keep = keep.replace("--", "-")
    return keep.strip("-")


class ApiError(RuntimeError):
    pass


class Client:
    def __init__(self, base_url: str):
        self.base_url = base_url.rstrip("/")

    def call(self, method: str, path: str, body: dict | None = None):
        url = f"{self.base_url}{path}"
        data = json.dumps(body).encode() if body is not None else None
        request = urllib.request.Request(url, data=data, method=method)
        request.add_header("Content-Type", "application/json")
        try:
            with urllib.request.urlopen(request, timeout=10) as response:
                raw = response.read()
                return json.loads(raw) if raw else None
        except urllib.error.HTTPError as error:
            detail = error.read().decode(errors="replace")
            raise ApiError(f"{method} {path} -> {error.code}: {detail}") from None
        except urllib.error.URLError as error:
            raise ApiError(f"Could not reach {url}: {error.reason}") from None


def make_note_body(task_title: str) -> str:
    bullets = random.sample(NOTE_BULLETS, k=random.randint(1, 3))
    lines = [f"## {task_title}", "", random.choice(NOTE_OPENERS), ""]
    lines += [f"- {bullet}" for bullet in bullets]
    return "\n".join(lines)


def random_session(now: datetime, day: int) -> tuple[datetime, datetime]:
    start = now.replace(
        day=day,
        hour=random.randint(WORKDAY_START_HOUR, WORKDAY_END_HOUR - 1),
        minute=random.randint(0, 59),
        second=0,
        microsecond=0,
    )
    if start > now:
        start = now - timedelta(minutes=random.randint(30, 240))
    stop = start + timedelta(minutes=random.randint(20, 150))
    if stop > now:
        stop = now
    if stop <= start:
        stop = start + timedelta(minutes=5)
    return start, stop


def iso(dt: datetime) -> str:
    return dt.strftime("%Y-%m-%dT%H:%M:%SZ")


def seed(client: Client, companies_count: int, tasks_per_company: int, force: bool) -> None:
    existing = client.call("GET", "/api/companies")
    if existing and not force:
        names = ", ".join(c["name"] for c in existing)
        print(f"{client.base_url} already has data: {names}", file=sys.stderr)
        print("Refusing to seed on top of it. Use --force to add anyway, or run against "
              "a clean instance (see `make reset-data` / `make run-demo`).", file=sys.stderr)
        sys.exit(1)

    pool = list(COMPANIES)
    if companies_count > len(pool):
        pool = pool * (companies_count // len(pool) + 1)
    chosen = pool[:companies_count]

    now = datetime.now()
    today = now.day

    for index, (name, description, project_words) in enumerate(chosen):
        company_name = name if index < len(COMPANIES) else f"{name} {index // len(COMPANIES) + 1}"
        company = client.call("POST", "/api/companies", {"name": company_name, "description": description})
        print(f"company: {company['name']}")

        project_count = random.randint(2, min(4, len(project_words)))
        projects = []
        for word in random.sample(project_words, k=project_count):
            project = client.call("POST", "/api/projects", {
                "label": word,
                "title": word.replace("-", " ").title(),
                "status": "ACTIVE",
                "description": None,
                "blueprintMarkdown": None,
                "companyId": company["id"],
            })
            projects.append(project)
        print(f"  {len(projects)} project(s): {', '.join(p['label'] for p in projects)}")

        titles = random.sample(TASK_TITLES, k=min(tasks_per_company, len(TASK_TITLES)))
        while len(titles) < tasks_per_company:
            titles.append(random.choice(TASK_TITLES))

        running_task_index = random.randrange(len(titles))

        for task_index, title in enumerate(titles):
            project = projects[task_index % len(projects)]
            status = random.choices(TASK_STATUSES, weights=TASK_STATUS_WEIGHTS, k=1)[0]

            task = client.call("POST", "/api/tasks", {
                "label": slugify(title),
                "title": title,
                "status": status,
                "description": None,
                "projectId": project["id"],
            })

            client.call("POST", "/api/documents", {
                "title": f"{slugify(title)}.md",
                "kind": random.choice(DOCUMENT_KINDS),
                "bodyMarkdown": make_note_body(title),
                "taskIds": [task["id"]],
            })

            leave_running = task_index == running_task_index
            session_count = random.randint(1, 3)

            for _ in range(session_count - (1 if leave_running else 0)):
                started = client.call("POST", f"/api/tasks/{task['id']}/time-entries/start")
                client.call("POST", f"/api/tasks/{task['id']}/time-entries/stop")
                start_dt, stop_dt = random_session(now, random.randint(1, today))
                client.call("PATCH", f"/api/time-entries/{started['id']}", {
                    "startedAt": iso(start_dt),
                    "stoppedAt": iso(stop_dt),
                })

            if leave_running:
                started = client.call("POST", f"/api/tasks/{task['id']}/time-entries/start")
                start_dt = now - timedelta(minutes=random.randint(15, 180))
                client.call("PATCH", f"/api/time-entries/{started['id']}", {
                    "startedAt": iso(start_dt),
                    "stoppedAt": None,
                })

        print(f"  {len(titles)} task(s), each with a note and time tracked this month "
              f"(task #{running_task_index + 1} left running)")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-url", default=os.environ.get("REKALL_URL", "http://localhost:8080"))
    parser.add_argument("--companies", type=int, default=int(os.environ.get("DEMO_COMPANIES", 3)))
    parser.add_argument("--tasks-per-company", type=int,
                         default=int(os.environ.get("DEMO_TASKS_PER_COMPANY", 20)))
    parser.add_argument("--seed", type=int, default=None, help="Random seed, for a repeatable run")
    parser.add_argument("--force", action="store_true",
                         help="Seed even if the instance already has companies")
    args = parser.parse_args()

    if args.seed is not None:
        random.seed(args.seed)

    client = Client(args.base_url)
    try:
        seed(client, args.companies, args.tasks_per_company, args.force)
    except ApiError as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)

    print(f"\nDone. {args.companies} companies seeded against {args.base_url}.")


if __name__ == "__main__":
    main()
