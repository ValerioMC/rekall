"""
Drives the real UI in a headless browser and fails on any console error or failed request.

Run against a deployed instance:

    make test-ui-e2e

The API tests assert paths written by hand, so a typo in the frontend's own client would slip
past them. This walks the screens instead, which is the only way to catch a route that renders
but calls the wrong endpoint, or a component that throws after mount.

Two rules learned the hard way:

- Selectors are `data-testid`, never styling classes. The first version of this file broke
  entirely when the UI was restyled, which told us nothing about whether the UI worked.
- Waits are on elements, never on network idle. Navigation is client side, so the fetch begins
  after the route has already rendered and networkidle reports quiet too early.

The throwaway entity it creates is named uniquely per run and removed at the end, so the
target can be run repeatedly against the same instance.
"""

import json
import os
import sys
import urllib.request
import uuid

from playwright.sync_api import sync_playwright

BASE = os.environ.get("REKALL_URL", "http://localhost:8080")
SCRATCH = "probe_" + uuid.uuid4().hex[:8]
SCRATCH_LABEL = "Probe " + SCRATCH[-4:]

problems: list[str] = []
steps: list[str] = []


def api(method: str, path: str, body: object | None = None):
    data = json.dumps(body).encode() if body is not None else None
    request = urllib.request.Request(
        BASE + path, data=data, method=method, headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(request) as response:
        raw = response.read()
        return json.loads(raw) if raw else None


def cleanup() -> bool:
    """Removes the throwaway entity and the table this run created for it."""
    for table in api("GET", "/api/meta/tables"):
        if table["physicalName"] == SCRATCH:
            api("DELETE", f"/api/meta/tables/{table['id']}")
            api("POST", "/api/meta/apply", {"backfillDefaults": {}, "confirmedDrops": [SCRATCH]})
            return True
    return False


def watch(page) -> None:
    page.on("console", lambda m: problems.append(f"console.{m.type}: {m.text}") if m.type == "error" else None)
    page.on("pageerror", lambda e: problems.append(f"pageerror: {e}"))
    page.on(
        "requestfailed",
        lambda r: problems.append(f"requestfailed: {r.method} {r.url} ({r.failure})"),
    )
    page.on(
        "response",
        lambda r: problems.append(f"HTTP {r.status}: {r.request.method} {r.url}") if r.status >= 400 else None,
    )


def step(message: str) -> None:
    steps.append(message)
    print(f"  [{len(steps):2}] {message}")


def testid(page, name: str):
    return page.get_by_test_id(name)


with sync_playwright() as playwright:
    browser = playwright.chromium.launch()
    page = browser.new_page(viewport={"width": 1440, "height": 900})
    watch(page)

    # --- Schema list
    page.goto(f"{BASE}/schema", wait_until="networkidle")
    page.wait_for_selector("[data-testid=entity-card]", timeout=10_000)
    step(f"schema list loaded, title={page.title()!r}, {testid(page, 'entity-card').count()} entity card(s)")
    assert testid(page, "entity-card").count() >= 3, "expected the entities created earlier"

    # --- Opening each entity: the regression that started this file
    for name in ["Project", "Environment", "Task"]:
        page.goto(f"{BASE}/schema", wait_until="networkidle")
        page.locator("[data-testid=entity-card]", has_text=name).first.click()
        page.wait_for_selector("[data-testid=field-row]", timeout=10_000)
        heading = page.locator("header h1").inner_text()
        rows = testid(page, "field-row").count()
        step(f"opened entity {name!r}: heading={heading!r}, {rows} field row(s)")
        assert heading == name, f"expected heading {name}, got {heading}"
        assert rows > 0, f"{name} shows no fields"

    # --- Relations render on the entity that has them
    page.goto(f"{BASE}/schema", wait_until="networkidle")
    page.locator("[data-testid=entity-card]", has_text="Task").first.click()
    page.wait_for_selector("[data-testid=relation-outgoing]", timeout=10_000)
    step(f"task shows {testid(page, 'relation-outgoing').count()} outgoing relation(s)")
    assert testid(page, "relation-outgoing").count() >= 2, "task belongs to a project and an environment"

    # --- Plan, with nothing pending
    page.goto(f"{BASE}/plan", wait_until="networkidle")
    page.wait_for_selector("[data-testid=empty-state], [data-testid=plan-statement]", timeout=10_000)
    step(f"plan screen loaded, nothing-to-do={testid(page, 'empty-state').count() > 0}")

    # --- Create an entity through the UI
    page.goto(f"{BASE}/schema", wait_until="networkidle")
    page.get_by_role("button", name="New entity").click()
    testid(page, "entity-label").fill(SCRATCH_LABEL)
    testid(page, "entity-plural").fill(SCRATCH_LABEL + "s")
    testid(page, "entity-description").fill("Entita temporanea di verifica")
    suggested = testid(page, "entity-physical-name").input_value()
    step(f"new entity form suggested the table name {suggested!r}")
    assert suggested == SCRATCH_LABEL.lower().replace(" ", "_"), f"slug suggestion wrong: {suggested}"
    # Typed explicitly so the cleanup at the end knows exactly what to remove.
    testid(page, "entity-physical-name").fill(SCRATCH)
    page.get_by_role("button", name="Define entity").click()
    page.wait_for_selector(f"[data-testid=entity-card]:has-text('{SCRATCH_LABEL}')", timeout=10_000)
    step(f"created entity, cards now {testid(page, 'entity-card').count()}")

    # --- Add a field to it
    page.locator("[data-testid=entity-card]", has_text=SCRATCH_LABEL).first.click()
    page.wait_for_selector("[data-testid=empty-state]", timeout=10_000)
    page.get_by_role("button", name="Add field").click()
    testid(page, "field-label").fill("Clone url")
    testid(page, "field-description").fill("URL di clone del repository")
    page.get_by_role("button", name="Add field").last.click()
    page.wait_for_selector("[data-testid=field-row]", timeout=10_000)
    step(f"added a field, table now has {testid(page, 'field-row').count()} row(s)")
    assert testid(page, "field-row").count() == 1

    # --- Set the identifying field
    testid(page, "set-identifier").first.click()
    # Waiting on the badge itself, not on the word: the "no identifier" warning shown before
    # the save also contains it, and matching that navigated away mid-request.
    page.wait_for_selector("[data-testid=identifier-badge]", timeout=10_000)
    step("identifying field set")

    # --- Apply the plan through the UI
    page.goto(f"{BASE}/plan", wait_until="networkidle")
    page.wait_for_selector("[data-testid=plan-statement]", timeout=10_000)
    statements = testid(page, "plan-statement").count()
    step(f"plan shows {statements} statement card(s), {testid(page, 'plan-sql').count()} with SQL")
    assert statements > 0, "creating an entity should produce a plan"
    assert testid(page, "plan-sql").count() > 0, "the plan must render the SQL"
    page.get_by_role("button", name="Apply").click()
    page.wait_for_selector("[data-testid=toast]", timeout=15_000)
    step(f"applied; toast={testid(page, 'toast').first.inner_text().splitlines()[0]!r}")

    # --- Data browser
    page.goto(f"{BASE}/data/project", wait_until="networkidle")
    page.wait_for_selector("[data-testid=record-row]", timeout=10_000)
    step(f"project data browser shows {testid(page, 'record-row').count()} record row(s)")

    # --- A record with resolved references and a document
    page.goto(f"{BASE}/data/task", wait_until="networkidle")
    page.wait_for_selector("[data-testid=record-link]", timeout=10_000)
    page.locator("[data-testid=record-link]", has_text="code-validator-main-workflow").first.click()
    page.wait_for_selector("[data-testid=record-references]", timeout=10_000)
    step("opened the task that has a document and two references")
    testid(page, "document-tab").first.click()
    page.wait_for_selector("[data-testid=document-preview]", timeout=10_000)
    preview = testid(page, "document-preview").inner_text()
    step(f"document preview rendered, {len(preview)} chars")
    assert "kmaster14" in preview, "markdown preview did not render the body"

    # --- Search
    page.goto(f"{BASE}/search", wait_until="networkidle")
    page.get_by_placeholder("cluster name, endpoint, anything you wrote down").fill("kmaster14")
    page.get_by_role("button", name="Search").click()
    page.wait_for_selector("[data-testid=search-result]", timeout=10_000)
    step(f"search returned {testid(page, 'search-result').count()} result card(s)")

    page.screenshot(path=os.environ.get("REKALL_SCREENSHOT", "/tmp/rekall-ui.png"), full_page=True)
    browser.close()

step(f"cleaned up the throwaway entity {SCRATCH!r}: {cleanup()}")

print()
if problems:
    print(f"{len(problems)} browser problem(s):")
    for problem in dict.fromkeys(problems):
        print("  -", problem)
    sys.exit(1)
print(f"All {len(steps)} UI steps passed with no console errors and no failed requests.")
