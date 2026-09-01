#!/usr/bin/env python3
"""Populates a running Rekall instance with demo companies, projects, tasks, briefs, notes
and time tracking for the current month.

Projects arrive with a description and a blueprint, tasks with a markdown brief of their own,
so the three markdown surfaces the console added (the project blueprint, the task description
and the note) all have something in the demo database worth reading. The briefs are written in
four shapes on rotation — a checklist, an approach, a constraints table, a set of pointers —
because a description is where the editor's syntax actually shows: headings, task lists,
tables, fenced code, quotes and links. Every seventh task is left without one, so the empty
state of the description card is on screen too.

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
from datetime import datetime, timedelta, timezone

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

# The brief of a task, in the shapes it is actually written in. The verb a title opens with
# says what kind of work it is, and the kind decides how the brief opens: a defect is argued
# from the report, a build from the scope, a cleanup from what disappears.
TASK_INTENTS = {
    "Fix": "defect",
    "Investigate": "defect",
    "Add": "build",
    "Implement": "build",
    "Set up": "build",
    "Design": "build",
    "Draft": "build",
    "Write": "build",
    "Refactor": "cleanup",
    "Clean up": "cleanup",
    "Simplify": "cleanup",
    "Consolidate": "cleanup",
    "Deprecate": "cleanup",
    "Improve": "change",
    "Optimize": "change",
    "Reduce": "change",
    "Harden": "change",
    "Upgrade": "change",
    "Update": "change",
    "Migrate": "change",
    "Rotate": "change",
}

INTENT_OPENERS = {
    "defect": [
        "Reproducible on the current build, and reported often enough that a workaround is not "
        "the answer. Done when the cause is understood, not when the symptom stops showing.",
        "Two reports this month, both arriving through the same path. The interesting part is "
        "why the state gets written twice, not the message the user ends up reading.",
    ],
    "build": [
        "Nothing covers this today. What is described below is the smallest version that is "
        "actually useful, and it is deliberately cheap to grow later.",
        "Talked through with the team, and the shape below is what we settled on. Build that "
        "one, not the more general thing standing behind it.",
    ],
    "cleanup": [
        "No behaviour change. When this lands the same idea lives in one place instead of "
        "three, and every test that passes now still passes.",
        "This has drifted once already. The point is to pick the version that stays and delete "
        "the rest in the same pass, so there is nothing left to drift from.",
    ],
    "change": [
        "It works today, badly enough to be worth an afternoon. The numbers below are what "
        "makes it done, not the impression that it got better.",
        "Straightforward on paper; the cost is in the call sites. The order below keeps every "
        "step shippable on its own, so it can stop halfway without leaving a mess.",
    ],
}

DONE_WHEN = [
    "A test fails without the change and passes with it",
    "The empty state and the loaded state both read correctly in the console",
    "The build log carries no new warning",
    "It ships behind a flag until the first canary is clean",
    "The blueprint of the project says the new rule out loud",
    "Nothing needs a migration window to roll out",
    "Measured before and after, both numbers written into the wrapup",
]

OUT_OF_SCOPE = [
    "The wider redesign of the same screen. Separate task, separate review.",
    "Anything that needs a schema migration. That lands on its own.",
    "Backfilling the rows already written. Tracked separately.",
    "The second call site. It gets the same treatment once this one is proven.",
]

APPROACH_STEPS = [
    "Reproduce it in a test first: red before green.",
    "Land the plumbing with the behaviour unchanged.",
    "Move the call sites over one at a time.",
    "Delete the old path once nothing points at it.",
    "Measure again and record the number in the wrapup.",
]

CONSTRAINTS = [
    ("No new dependency", "The bundle budget is already spent"),
    ("Backward compatible for one release", "Older clients update on their own schedule"),
    ("Under 200 ms at p95", "This sits on the render path"),
    ("Idempotent", "The caller retries on every 5xx"),
    ("Works with no network", "The desktop build has no connectivity guarantee"),
]

# What a project hands to whoever opens it, `/rk project:<label>` included. Kept short on
# purpose: a blueprint nobody finishes reading is a blueprint nobody reads.
PROJECT_DESCRIPTIONS = [
    "Owns {subject} end to end, from the API contract down to the storage under it.",
    "Everything {company} ships through {subject}: the service, its client, and the runbook "
    "behind them.",
    "The {subject} surface. Small on purpose, and the one place the rules about it are written.",
]

PROJECT_STACK = [
    "Spring Boot service, H2 locally and Postgres in staging, schema owned by Flyway",
    "Vue 3 and Vite on the front, Pinia for state, no component library",
    "Contract first: the OpenAPI document is the source and the client is generated from it",
    "Everything asynchronous goes through one queue; no service calls a peer synchronously",
    "One deploy unit, so there is no worker release to keep in step",
]

PROJECT_CONVENTIONS = [
    "Branches `feat/<ticket>-short-desc`, commit messages in Conventional Commits",
    "Controller to service to repository; a controller never touches a repository",
    "Errors are typed and explicit, and nothing returns `null` to mean failure",
    "Structured logging only, and every line carries the request id",
    "A public endpoint is not merged without an integration test against it",
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


def intent_of(title: str) -> str:
    """The kind of work a title announces, read off the verb it opens with."""
    for verb, intent in TASK_INTENTS.items():
        if title.startswith(verb):
            return intent
    return "build"


def shape_checklist(label: str, opener: str, status: str) -> str:
    """A brief argued as acceptance: what has to be true before this is closed."""
    items = random.sample(DONE_WHEN, k=3)
    done_count = len(items) if status == "DONE" else random.randint(0, 1)
    lines = [opener, "", "## Done when", ""]
    lines += [f"- [{'x' if index < done_count else ' '}] {item}" for index, item in enumerate(items)]
    lines += ["", "## Out of scope", "", random.choice(OUT_OF_SCOPE)]
    return "\n".join(lines)


def shape_approach(label: str, opener: str, status: str) -> str:
    """A brief argued as a route: the order that keeps every step shippable."""
    # Sampled by position and put back in order: the steps only read as a route if they stay
    # in the order they were written in, and "measure again" first reads as nonsense.
    steps = [APPROACH_STEPS[index] for index in sorted(random.sample(range(len(APPROACH_STEPS)), k=3))]
    lines = [opener, "", "## Approach", ""]
    lines += [f"{number}. {step}" for number, step in enumerate(steps, start=1)]
    lines += ["", "Check it the way CI does, before asking anyone to look:", "",
              "```bash", "make test-backend", "make test-ui", "```"]
    return "\n".join(lines)


def shape_constraints(label: str, opener: str, status: str) -> str:
    """A brief argued from its limits: the things that are not negotiable, and why."""
    rows = random.sample(CONSTRAINTS, k=3)
    lines = [opener, "", "## Constraints", "", "| Constraint | Why |", "| --- | --- |"]
    lines += [f"| {name} | {why} |" for name, why in rows]
    lines += ["", f"> Anchor `task:{label}`. What gets *learned* on the way belongs in a note; "
              "this stays the brief."]
    return "\n".join(lines)


def shape_pointers(label: str, opener: str, status: str) -> str:
    """A brief argued from the map: what this touches and where to read up first."""
    return "\n".join([
        opener,
        "",
        "**What it touches**",
        "",
        f"- `{label}`, and the two endpoints sitting in front of it",
        "- The console pane that reads it, which still assumes the field is never empty",
        "- The export, which carries the same field into every context it builds",
        "",
        f"Background in [the RFC](https://example.internal/rfc/{label}). "
        "*The public contract does not change.*",
    ])


DESCRIPTION_SHAPES = [shape_checklist, shape_approach, shape_constraints, shape_pointers]


def make_task_description(title: str, label: str, status: str, shape_index: int) -> str:
    """The markdown brief of one task, in the shape this position in the rotation calls for."""
    opener = random.choice(INTENT_OPENERS[intent_of(title)])
    shape = DESCRIPTION_SHAPES[shape_index % len(DESCRIPTION_SHAPES)]
    return shape(label, opener, status)


def make_project_description(title: str, company_name: str) -> str:
    """The few sentences a project carries into every context that loads it."""
    return random.choice(PROJECT_DESCRIPTIONS).format(subject=title.lower(), company=company_name)


def make_project_blueprint(title: str, label: str, company_name: str, description: str) -> str:
    """The blueprint `/rk project:<label>` hands over: what this is, and how work is done in it."""
    lines = [f"# {title}", "", description, "", "## How it is built", ""]
    lines += [f"- {item}" for item in random.sample(PROJECT_STACK, k=3)]
    lines += [
        "",
        "## Where things are",
        "",
        "| Path | What lives there |",
        "| --- | --- |",
        f"| `{label}/api` | HTTP in and out, and nothing else |",
        f"| `{label}/service` | The rules, in the only place they are written |",
        f"| `{label}/repository` | Storage, behind an interface |",
        "",
        "## Conventions",
        "",
    ]
    lines += [f"- {item}" for item in random.sample(PROJECT_CONVENTIONS, k=3)]
    lines += [
        "",
        "## Working in it",
        "",
        f"Load the context before touching anything of {company_name}:",
        "",
        "```bash",
        f"/rk project:{label}",
        "```",
    ]
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
    """UTC, which is the only thing a trailing `Z` may mean.

    Times are generated in the local zone so working hours read as working hours in the
    console; stamping those naive local times with a `Z` shifts every session by the local
    offset, and a running one started "now" lands in the future, where its live clock sits at
    zero until the wall clock catches up.
    """
    return dt.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


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

    now = datetime.now().astimezone()
    today = now.day

    for index, (name, description, project_words) in enumerate(chosen):
        company_name = name if index < len(COMPANIES) else f"{name} {index // len(COMPANIES) + 1}"
        company = client.call("POST", "/api/companies", {"name": company_name, "description": description})
        print(f"company: {company['name']}")

        project_count = random.randint(2, min(4, len(project_words)))
        projects = []
        for word in random.sample(project_words, k=project_count):
            project_title = word.replace("-", " ").title()
            project_description = make_project_description(project_title, company_name)
            project = client.call("POST", "/api/projects", {
                "label": word,
                "title": project_title,
                "status": "ACTIVE",
                "description": project_description,
                "blueprintMarkdown": make_project_blueprint(
                    project_title, word, company_name, project_description),
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

            label = slugify(title)
            # Every seventh task stays undescribed: the description card has an empty state,
            # and a demo database where it never appears hides half of what that card does.
            described = task_index % 7 != 6

            task = client.call("POST", "/api/tasks", {
                "label": label,
                "title": title,
                "status": status,
                "description": make_task_description(title, label, status, task_index)
                if described else None,
                "projectId": project["id"],
            })

            client.call("POST", "/api/documents", {
                "title": f"{label}.md",
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

        described_count = sum(1 for index in range(len(titles)) if index % 7 != 6)
        print(f"  {len(titles)} task(s), each with a note and time tracked this month "
              f"({described_count} with a markdown brief, "
              f"task #{running_task_index + 1} left running)")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-url", default=os.environ.get("REKALL_URL", "http://localhost:47355"))
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
