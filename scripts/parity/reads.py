"""Read everything a server holds, with nothing normalised but the port: for comparing the Java
server with the Rust one running on the same data (a Rust database imported from the Java one)."""
import io, json, sys, urllib.request, urllib.error, zipfile
BASE, OUT = sys.argv[1], sys.argv[2]
record = []
def call(label, method, path, body=None, headers=None):
    data = json.dumps(body).encode() if body is not None else None
    h = dict(headers or {})
    if data is not None: h["Content-Type"] = "application/json"
    req = urllib.request.Request(BASE + path, data=data, method=method, headers=h)
    try:
        with urllib.request.urlopen(req, timeout=30) as r: status, payload = r.status, r.read()
    except urllib.error.HTTPError as e: status, payload = e.code, e.read()
    entry = {"label": label, "status": status}
    if payload[:2] == b"PK":
        z = zipfile.ZipFile(io.BytesIO(payload))
        entry["zip"] = {n: z.read(n).decode("utf-8", "replace") for n in sorted(z.namelist())}
    else:
        try: entry["body"] = json.loads(payload)
        except Exception: entry["text"] = payload.decode("utf-8", "replace")[:500]
    record.append(entry)
    return entry.get("body")
def tool(label, name, args):
    return call(label, "POST", "/mcp", {"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": name, "arguments": args}})
companies = call("companies", "GET", "/api/companies")
projects = call("projects", "GET", "/api/projects")
tasks = call("tasks", "GET", "/api/tasks")
for p in projects or []:
    call("project " + p["label"], "GET", f"/api/projects/{p['id']}")
    call("tasks of " + p["label"], "GET", f"/api/tasks?projectId={p['id']}")
    tool("ctx project " + p["label"], "rekall_context", {"anchors": p["anchor"]})
for c in companies or []:
    tool("ctx company " + c["name"], "rekall_context", {"anchors": "company:" + c["name"]})
for t in tasks or []:
    call("task " + t["label"], "GET", f"/api/tasks/{t['id']}")
    call("steps of " + t["label"], "GET", f"/api/tasks/{t['id']}/steps")
    call("wrapup of " + t["label"], "GET", f"/api/tasks/{t['id']}/wrapup")
    call("docs of " + t["label"], "GET", f"/api/documents?taskId={t['id']}")
    call("size of " + t["label"], "GET", f"/api/tasks/{t['id']}/context-size")
    call("revisions of " + t["label"], "GET", f"/api/tasks/{t['id']}/revisions")
    tool("ctx task " + t["label"], "rekall_context", {"anchors": t["anchor"]})
docs = call("documents", "GET", "/api/documents")
for d in docs or []:
    tool("ctx note " + d["title"], "rekall_context", {"anchors": d["anchor"]})
call("steps", "GET", "/api/steps")
call("wrapups", "GET", "/api/wrapups")
call("time entries", "GET", "/api/time-entries")
call("commit refs", "GET", "/api/commit-references")
call("tags", "GET", "/api/tags")
call("run queue", "GET", "/api/run-queue")
call("search", "GET", "/api/search?q=the")
call("search bastion", "GET", "/api/search?q=bastion")
call("doc search", "GET", "/api/documents/search?q=a")
call("export", "GET", "/api/export")
json.dump(record, open(OUT, "w"), indent=1, ensure_ascii=False)
print(len(record), "reads")
