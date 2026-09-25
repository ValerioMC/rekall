"""Drive one Rekall server through the same sequence of calls and record every answer, normalised
so two servers (the Java one and the Rust one) can be diffed: ids, timestamps, ports and the
per-run folders are replaced by placeholders in order of first appearance."""

import io
import json
import re
import sys
import urllib.error
import urllib.request
import uuid
import zipfile

BASE = sys.argv[1]
REPO = sys.argv[2]
PLAIN = sys.argv[3]
OUT = sys.argv[4]
KEEP = "--keep" in sys.argv  # leave the data in place, for reads.py to compare after an import
FOLDERS = [a for a in sys.argv[5:] if a != "--keep"]  # per-run paths to hide

record = []
ids = {}


def norm(text):
    if text is None:
        return None
    for i, folder in enumerate(FOLDERS):
        text = text.replace(folder, f"<run-folder-{i}>")
    text = text.replace(BASE.split(":")[-1], "<port>")

    def uid(m):
        key = m.group(0).lower()
        if key not in ids:
            ids[key] = f"<id-{len(ids) + 1}>"
        return ids[key]

    text = re.sub(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}", uid, text)
    text = re.sub(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z", "<ts>", text)
    text = re.sub(r"rekall-\d{8}-\d{6}(-\d+)?-", "rekall-<stamp>-", text)
    text = re.sub(r"note:[0-9a-f]{8}\b", "note:<prefix>", text)
    return text


def call(label, method, path, body=None, headers=None, raw=None):
    data = None
    hdrs = dict(headers or {})
    if raw is not None:
        data = raw.encode()
        hdrs.setdefault("Content-Type", "application/json")
    elif body is not None:
        data = json.dumps(body).encode()
        hdrs["Content-Type"] = "application/json"
    request = urllib.request.Request(BASE + path, data=data, method=method, headers=hdrs)
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            status, ctype, payload = response.status, response.headers.get("Content-Type", ""), response.read()
    except urllib.error.HTTPError as e:
        status, ctype, payload = e.code, e.headers.get("Content-Type", ""), e.read()
    text = payload.decode("utf-8", "replace")
    parsed = None
    try:
        parsed = json.loads(text)
    except Exception:
        pass
    entry = {"label": label, "call": f"{method} {norm(path)}", "status": status, "type": ctype.split(";")[0]}
    if parsed is not None:
        entry["body"] = json.loads(norm(json.dumps(parsed, ensure_ascii=False)))
    elif payload[:2] == b"PK":
        archive = zipfile.ZipFile(io.BytesIO(payload))
        entry["zip"] = {norm(n): norm(archive.read(n).decode("utf-8", "replace")) for n in sorted(archive.namelist())}
    else:
        entry["text"] = norm(text[:300])
    record.append(entry)
    return parsed


def mcp(label, method, params=None, id_=1, headers=None):
    body = {"jsonrpc": "2.0", "method": method}
    if id_ is not None:
        body["id"] = id_
    if params is not None:
        body["params"] = params
    return call(label, "POST", "/mcp", body, headers)


def tool(label, name, arguments):
    return mcp(label, "tools/call", {"name": name, "arguments": arguments})


def id_of(value):
    return (value or {}).get("id", str(uuid.uuid4()))


call("health", "GET", "/actuator/health")
call("settings status", "GET", "/api/settings/databases")
call("settings check", "GET", f"/api/settings/databases/check?path={PLAIN}")
call("settings check no param", "GET", "/api/settings/databases/check")
call("claude usage", "GET", "/api/claude/usage")
call("claude usage bad refresh", "GET", "/api/claude/usage?refresh=maybe")
call("companies empty", "GET", "/api/companies")
acme = call("company create", "POST", "/api/companies", {"name": "Acme", "description": "Il cliente"})
call("company duplicate", "POST", "/api/companies", {"name": "Acme"})
call("company no name", "POST", "/api/companies", {})
call("company blank name", "POST", "/api/companies", {"name": "  "})
call("company bad json", "POST", "/api/companies", raw="{nope")
call("company empty body", "POST", "/api/companies", raw="")
globex = call("company 2", "POST", "/api/companies", {"name": "Globex"})
call("company update", "PUT", f"/api/companies/{id_of(globex)}", {"name": "Globex Corp", "description": "x"})
call("company update unknown", "PUT", f"/api/companies/{uuid.uuid4()}", {"name": "Nobody"})
call("company bad uuid", "PUT", "/api/companies/not-a-uuid", {"name": "Nobody"})
call("companies", "GET", "/api/companies")

vega = call("project create", "POST", "/api/projects", {"label": "Vega Platform", "title": "Vega", "status": "ACTIVE", "companyId": id_of(acme), "repoFolder": REPO, "description": "Progetto"})
beacon = call("project 2", "POST", "/api/projects", {"label": "beacon", "title": "Beacon", "status": "PAUSED", "companyId": id_of(acme)})
call("project bad label", "POST", "/api/projects", {"label": "///", "title": "x", "status": "ACTIVE", "companyId": id_of(acme)})
call("project no company", "POST", "/api/projects", {"label": "x", "title": "x", "status": "ACTIVE"})
call("project unknown company", "POST", "/api/projects", {"label": "x", "title": "x", "status": "ACTIVE", "companyId": str(uuid.uuid4())})
call("project bad status", "POST", "/api/projects", {"label": "x", "title": "x", "status": "NOPE", "companyId": id_of(acme)})
call("project no title", "POST", "/api/projects", {"label": "x", "status": "ACTIVE", "companyId": id_of(acme)})
call("project duplicate label", "POST", "/api/projects", {"label": "beacon", "title": "x", "status": "ACTIVE", "companyId": id_of(acme)})
call("project long title", "POST", "/api/projects", {"label": "long", "title": "x" * 300, "status": "ACTIVE", "companyId": id_of(acme)})
call("projects", "GET", "/api/projects")
call("project get", "GET", f"/api/projects/{id_of(vega)}")
call("project get unknown", "GET", f"/api/projects/{uuid.uuid4()}")
call("project repository", "GET", f"/api/projects/{id_of(vega)}/repository")
call("project repository plain", "GET", f"/api/projects/{id_of(beacon)}/repository")
call("project update", "PUT", f"/api/projects/{id_of(beacon)}", {"label": "beacon", "title": "Beacon 2", "status": "DONE", "companyId": id_of(acme), "repoFolder": PLAIN, "autoCommit": True, "blueprintMarkdown": "# Blueprint"})

t1 = call("task create", "POST", "/api/tasks", {"label": "report-builder", "title": "Report builder", "status": "IN_PROGRESS", "projectId": id_of(vega), "description": "## Goal\n\nBuild it."})
t2 = call("task 2", "POST", "/api/tasks", {"label": "retry-policy", "title": "Retry policy", "status": "TODO", "projectId": id_of(vega)})
t3 = call("task 3", "POST", "/api/tasks", {"label": "setup", "title": "Setup", "status": "BLOCKED", "projectId": id_of(beacon)})
call("task duplicate", "POST", "/api/tasks", {"label": "Report-Builder", "title": "Again", "status": "TODO", "projectId": id_of(vega)})
call("task no title", "POST", "/api/tasks", {"label": "x", "status": "TODO", "projectId": id_of(vega)})
call("task bad status", "POST", "/api/tasks", {"label": "x", "title": "x", "status": "OPEN", "projectId": id_of(vega)})
call("tasks", "GET", "/api/tasks")
call("tasks of project", "GET", f"/api/tasks?projectId={id_of(vega)}")
call("task get", "GET", f"/api/tasks/{id_of(t1)}")
call("task update", "PUT", f"/api/tasks/{id_of(t2)}", {"label": "retry-policy", "title": "Retry policy v2", "status": "IN_PROGRESS", "projectId": id_of(vega), "description": "desc"})

tag = call("tag create", "POST", "/api/tags", {"name": "urgent", "color": "#ff0000"})
call("tag duplicate", "POST", "/api/tags", {"name": "urgent"})
call("tags", "GET", "/api/tags")
call("tag update", "PUT", f"/api/tags/{id_of(tag)}", {"name": "later"})

d1 = call("doc create", "POST", "/api/documents", {"title": "CONTEXT.md", "kind": "context", "taskIds": [id_of(t1)], "bodyMarkdown": "# Contesto\n\nIl workflow parte da POST /api/v1/pipelines."})
d2 = call("doc shared", "POST", "/api/documents", {"title": "kmaster14.md", "kind": "notes", "taskIds": [id_of(t1), id_of(t2)], "bodyMarkdown": "Accesso via bastion."})
call("doc no tasks", "POST", "/api/documents", {"title": "x", "kind": "notes", "taskIds": []})
call("docs", "GET", "/api/documents")
call("docs of task", "GET", f"/api/documents?taskId={id_of(t2)}")
call("docs search", "GET", "/api/documents/search?q=bastion")
call("doc update reference", "PUT", f"/api/documents/{id_of(d2)}", {"title": "kmaster14.md", "kind": "notes", "taskIds": [id_of(t1), id_of(t2)], "bodyMarkdown": "Accesso via bastion.\n\nDettaglio.", "contextMode": "REFERENCE"})

s1 = call("step add", "POST", f"/api/tasks/{id_of(t1)}/steps", {"title": "Aggregate the rows", "bodyMarkdown": "Somma per settimana."})
s2 = call("step add 2", "POST", f"/api/tasks/{id_of(t1)}/steps", {"title": "Write the tests"})
call("step add blank", "POST", f"/api/tasks/{id_of(t1)}/steps", {"title": " "})
call("step promote", "PATCH", f"/api/steps/{id_of(s1)}", {"draft": False})
call("step promote 2", "PATCH", f"/api/steps/{id_of(s2)}", {"draft": False})
call("step move", "POST", f"/api/steps/{id_of(s2)}/move", {"position": 0})
call("steps of task", "GET", f"/api/tasks/{id_of(t1)}/steps")
call("steps", "GET", "/api/steps")

call("wrapup by hand", "PUT", f"/api/tasks/{id_of(t2)}/wrapup", {"bodyMarkdown": "Scritto a mano."})
call("wrapup get", "GET", f"/api/tasks/{id_of(t2)}/wrapup")
call("wrapup get none", "GET", f"/api/tasks/{id_of(t1)}/wrapup")

mcp("mcp initialize", "initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "parity", "version": "1"}})
mcp("mcp initialized", "notifications/initialized", None, id_=None)
mcp("mcp ping", "ping", {})
mcp("mcp tools/list", "tools/list", {})
mcp("mcp unknown method", "nope/nope", {})
call("mcp bad jsonrpc", "POST", "/mcp", {"jsonrpc": "1.0", "id": 1, "method": "ping"})
call("mcp not json", "POST", "/mcp", raw="{bad")
call("mcp get", "GET", "/mcp")
tool("ctx project", "rekall_context", {"anchors": "project:vega-platform"})
tool("ctx task", "rekall_context", {"anchors": "project:vega-platform task:report-builder"})
tool("ctx company", "rekall_context", {"anchors": "company:Acme"})
tool("ctx ambiguous", "rekall_context", {"anchors": "setup"})
tool("ctx unknown", "rekall_context", {"anchors": "task:nowhere"})
tool("ctx empty", "rekall_context", {"anchors": ""})
tool("step running", "rekall_step", {"anchors": "project:vega-platform task:report-builder", "step": "1", "state": "running"})
tool("step claimed", "rekall_step", {"anchors": "project:vega-platform task:report-builder", "step": "Aggregate the rows", "state": "claimed"})
tool("step done", "rekall_step", {"anchors": "project:vega-platform task:report-builder", "step": "1", "state": "done"})
tool("step bad state", "rekall_step", {"anchors": "project:vega-platform task:report-builder", "step": "1", "state": "flying"})
tool("step unknown", "rekall_step", {"anchors": "project:vega-platform task:report-builder", "step": "9", "state": "running"})
tool("propose", "rekall_propose_step", {"anchors": "project:vega-platform task:report-builder", "title": "Expose the download", "detail": "Stream it."})
tool("propose dup", "rekall_propose_step", {"anchors": "project:vega-platform task:report-builder", "title": "expose the download"})
tool("wrapup", "rekall_wrapup", {"anchors": "project:vega-platform task:report-builder", "body": "## Stato\n\nLe righe sono aggregate."})
tool("wrapup replace hand", "rekall_wrapup", {"anchors": "project:vega-platform task:retry-policy", "body": "Riscritto."})
tool("wrapup too long", "rekall_wrapup", {"anchors": "project:vega-platform task:report-builder", "body": "x" * 20001})
tool("record commit", "rekall_record_commit", {"anchors": "project:vega-platform task:report-builder"})
tool("record commit step", "rekall_record_commit", {"anchors": "project:vega-platform task:report-builder", "step": "1"})
tool("record commit bad hash", "rekall_record_commit", {"anchors": "project:vega-platform task:report-builder", "commit": "deadbeef"})
tool("record commit no folder", "rekall_record_commit", {"anchors": "project:beacon task:setup"})
tool("unknown tool", "rekall_nothing", {})
tool("ctx after", "rekall_context", {"anchors": "project:vega-platform task:report-builder"})
modern = {"MCP-Protocol-Version": "2026-07-28", "Mcp-Method": "tools/call", "Mcp-Name": "rekall_context"}
mcp("modern call", "tools/call", {"name": "rekall_context", "arguments": {"anchors": "project:vega-platform"}}, headers=modern)
mcp("modern mismatch", "tools/call", {"name": "rekall_step", "arguments": {}}, headers=modern)
mcp("modern discover", "server/discover", {}, headers={"MCP-Protocol-Version": "2026-07-28", "Mcp-Method": "server/discover"})
mcp("modern tools/list", "tools/list", {}, headers={"MCP-Protocol-Version": "2026-07-28", "Mcp-Method": "tools/list"})

call("wrapups", "GET", "/api/wrapups")
call("step done console", "PATCH", f"/api/steps/{id_of(s1)}", {"done": True})
call("review accept stepless", "PATCH", f"/api/tasks/{id_of(t3)}/review", {"reviewState": "DONE"})
call("review again", "PATCH", f"/api/tasks/{id_of(t3)}/review", {"reviewState": "DONE"})
call("review bad", "PATCH", f"/api/tasks/{id_of(t3)}/review", {"reviewState": "CLAIMED"})

e1 = call("timer start", "POST", f"/api/tasks/{id_of(t1)}/time-entries/start")
call("timer start again", "POST", f"/api/tasks/{id_of(t1)}/time-entries/start")
call("timer stop", "POST", f"/api/tasks/{id_of(t1)}/time-entries/stop")
call("timer stop none", "POST", f"/api/tasks/{id_of(t1)}/time-entries/stop")
call("time entries", "GET", "/api/time-entries")
call("time entry edit bad", "PATCH", f"/api/time-entries/{id_of(e1)}", {"startedAt": "2030-01-01T00:00:00Z", "stoppedAt": "2020-01-01T00:00:00Z"})
call("time entry edit", "PATCH", f"/api/time-entries/{id_of(e1)}", {"startedAt": "2026-01-01T09:00:00Z", "stoppedAt": "2026-01-01T10:30:00Z"})
call("time entry edit missing", "PATCH", f"/api/time-entries/{id_of(e1)}", {})

latest = call("commit latest", "POST", f"/api/tasks/{id_of(t2)}/commit-references/latest")
call("recent commits", "GET", f"/api/tasks/{id_of(t2)}/recent-commits")
call("commit by hash", "POST", f"/api/tasks/{id_of(t2)}/commit-references", {"commitHash": "HEAD~1"})
call("commit no hash", "POST", f"/api/tasks/{id_of(t2)}/commit-references", {})
call("commit refs", "GET", "/api/commit-references")
call("commit diff", "GET", f"/api/commit-references/{id_of(latest)}/diff")
call("commit in context", "PATCH", f"/api/commit-references/{id_of(latest)}", {"inContext": True})
tool("ctx with commit", "rekall_context", {"anchors": "project:vega-platform task:retry-policy"})
call("commit delete", "DELETE", f"/api/commit-references/{id_of(latest)}")
call("commit delete again", "DELETE", f"/api/commit-references/{id_of(latest)}")

call("search", "GET", "/api/search?q=bastion")
call("search blank", "GET", "/api/search?q=")
call("context size", "GET", f"/api/tasks/{id_of(t1)}/context-size")
revs = call("revisions", "GET", f"/api/tasks/{id_of(t2)}/revisions?kind=WRAPUP")
call("revisions all", "GET", f"/api/tasks/{id_of(t2)}/revisions")
if isinstance(revs, list) and revs:
    call("revision restore", "POST", f"/api/tasks/{id_of(t2)}/revisions/{revs[0]['id']}/restore", {})
call("export", "GET", "/api/export")

call("queue", "GET", "/api/run-queue")
call("queue settings", "PUT", "/api/run-queue/settings", {"ceilingPercent": 80, "skipPermissions": True, "model": "sonnet", "effort": "high"})
call("queue settings bad", "PUT", "/api/run-queue/settings", {"ceilingPercent": 140})
call("queue settings bad model", "PUT", "/api/run-queue/settings", {"model": "gpt"})
i1 = call("queue add", "POST", "/api/run-queue/items", {"taskId": id_of(t1)})
call("queue add 2", "POST", "/api/run-queue/items", {"taskId": id_of(t2)})
call("queue add dup", "POST", "/api/run-queue/items", {"taskId": id_of(t1)})
call("queue add none", "POST", "/api/run-queue/items", {})
q = call("queue view", "GET", "/api/run-queue")
second = (q or {}).get("items", [{}, {}])[-1].get("id", str(uuid.uuid4()))
call("queue move", "PUT", f"/api/run-queue/items/{second}/position", {"index": 0})
call("queue move none", "PUT", f"/api/run-queue/items/{second}/position", {})
call("queue start past", "POST", "/api/run-queue/start", {"startAt": "2020-01-01T00:00:00Z"})
call("queue start later", "POST", "/api/run-queue/start", {"startAt": "2099-01-01T00:00:00Z"})
call("queue stop", "POST", "/api/run-queue/stop")
call("queue remove", "DELETE", f"/api/run-queue/items/{second}")
call("queue clear", "POST", "/api/run-queue/clear")

call("terminals", "GET", "/api/terminals")
call("terminal no folder", "POST", f"/api/tasks/{id_of(t3)}/terminals", {"skipPermissions": True})
call("terminal unknown", "GET", f"/api/terminals/{uuid.uuid4()}")
call("claude settings", "GET", "/api/settings/claude")
call("backups", "GET", "/api/backups")
call("backup bad name", "GET", "/api/backups/nope.zip")

call("unknown api", "GET", "/api/nope")
call("method not allowed", "DELETE", "/api/companies")
call("post unknown", "POST", "/nope", {})
call("spa root", "GET", "/")
call("spa deep", "GET", "/projects/abc")
call("static missing", "GET", "/assets/missing.js")
call("foreign origin", "GET", "/api/companies", headers={"Origin": "https://evil.example"})
call("foreign host", "GET", "/api/companies", headers={"Host": "evil.example"})

if KEEP:
    json.dump(record, open(OUT, "w"), indent=1, ensure_ascii=False)
    print(f"{len(record)} calls recorded")
    sys.exit(0)

call("step delete", "DELETE", f"/api/steps/{id_of(s2)}")
call("tag delete", "DELETE", f"/api/tags/{id_of(tag)}")
call("doc delete", "DELETE", f"/api/documents/{id_of(d1)}")
call("wrapup delete", "DELETE", f"/api/tasks/{id_of(t2)}/wrapup")
call("task delete", "DELETE", f"/api/tasks/{id_of(t2)}")
call("project delete", "DELETE", f"/api/projects/{id_of(beacon)}")
call("company delete", "DELETE", f"/api/companies/{id_of(acme)}")
call("company delete again", "DELETE", f"/api/companies/{id_of(acme)}")
call("final companies", "GET", "/api/companies")

with open(OUT, "w") as f:
    json.dump(record, f, indent=1, ensure_ascii=False, sort_keys=False)
print(f"{len(record)} calls recorded")
