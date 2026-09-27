# Semantic Graph

The structured language a session uses to describe what a piece of software does. Rekall stores it as it arrives and draws it with its own code. A session never produces HTML, SVG, Vue or any other presentation, only this document.

The types, the reader and the rules live in `rekall-diagram` (`SemanticGraph`, `GraphCodec`, `GraphValidator`, `TraceIndex`). Every rule below is one the validator enforces, unless it is marked as guidance.

## The document

```json
{
  "format": "rekall.semantic-graph",
  "version": 1,
  "nodes": [ ... ],
  "edges": [ ... ]
}
```

- `format` must be `rekall.semantic-graph` and `version` must be `1`. Anything else is refused before any other rule is checked.
- No other fields are accepted, at any level. A field the format does not define is refused rather than dropped, so a typo cannot pass silently.
- A graph holds between 1 and 600 nodes and at most 2400 edges.

## Nodes

| Field | Required | Meaning |
|---|---|---|
| `id` | yes | Unique among nodes, at most 120 characters, no whitespace. Edges refer to it |
| `kind` | yes | See the kinds below |
| `title` | yes | Not blank, at most 200 characters. What the reader sees on the canvas: a short verb phrase or name |
| `description` | no | At most 4000 characters. What the title cannot say |
| `sources` | no | Where in the code this element lives. See Sources |
| `provenance` | no | `observed`, `inferred`, `documented` or `stated` |
| `confidence` | no | A number from 0 to 1 |
| `metadata` | no | Free key/value facts, shown in the inspector as they are |

### Kinds

| Kind | Use it for | Drawn as |
|---|---|---|
| `concept` | An idea or rule the code carries without naming it ("one live session per task") | Soft rounded card, teal |
| `action` | Something the system does ("Validate order") | Rounded card, blue |
| `decision` | A branch: the question that is asked ("Payment accepted?") | Lozenge, magenta |
| `state` | A condition the system rests in ("Step RUNNING") | Double-ringed pill, violet |
| `event` | Something that happens and starts or announces work ("Order received") | Signal arrow, orange |
| `data` | What is read or written: a table, a cache, a map in memory, a file | Cylinder, green |
| `external_system` | Anything outside this codebase: an API, a CLI, a database server | Barred box, slate |
| `code` | A real function, method, type or module | Code frame, grey |

Any other kind is accepted if it is snake_case (`^[a-z][a-z0-9_]{0,39}$`), such as `queue`. It is drawn as a generic card. Prefer the eight above.

## Edges

| Field | Required | Meaning |
|---|---|---|
| `from`, `to` | yes | Ids of existing nodes |
| `relation` | yes | See the relations below |
| `id` | no | Unique among edges. Edges that arrive without one are numbered `e1`, `e2`, … |
| `label` | no | What the relation alone does not say. On `conditionally_leads_to` this is the condition ("no", "card", "full") |
| `provenance`, `confidence`, `metadata` | no | As on nodes |

### Relations

| Relation | Reads as | Family on the canvas |
|---|---|---|
| `leads_to` | then | flow (solid) |
| `conditionally_leads_to` | if `label` | condition (dashed, carries the label) |
| `transitions_to` | becomes (between states, or from an action into a state) | flow |
| `triggers` | an event starts something | flow |
| `calls` | invokes (code to code, or to an external system) | flow |
| `reads`, `consumes` | takes from data | data (dotted) |
| `writes`, `produces` | puts into data | data |
| `depends_on` | needs, is governed by | structure (faint) |
| `contains` | holds: the parent is `from`, the child is `to` | never drawn as a line; it makes frames |

Any other snake_case relation is accepted and drawn as structure.

Two edges may not have the same `from`, `to`, `relation` and `label`. Two branches of one decision to the same target need different labels.

## Containment

`contains` has to form a forest:

- a node never contains itself,
- a node has at most one container,
- following containers upwards always ends; there are no cycles.

This is how a conceptual breakdown is tied to real structure: a `code` node for `process_order()` `contains` the actions and decisions it performs. In the **Code** lens the console draws the function as a frame around them. In the **Concept** lens it hides the code node and shows the pieces on their own. A container can be folded, and while folded it takes over its children's relations.

## Sources

```json
{ "file": "src/order.rs", "startLine": 42, "endLine": 61, "symbol": "process_order" }
```

- `file` is required and not blank. It is relative to the project's folder (`./` and `\` are normalised when compared). The console reads code only inside that folder, and any path that leaves it is refused.
- Lines are one-based and inclusive. `startLine` 0 is refused. An `endLine` needs a `startLine` and cannot come before it. Without lines, the element claims the whole file.
- `symbol` names the enclosing function or type when there is one.
- A node may list several sources (a concept spread across files). A node with none is a pure concept, and the trace mark shows a hollow core.
- A diagram stored through `rekall_diagram` has every span held against the project's folder before it is written: the file has to be there and `startLine` and `endLine` inside it. A span that points at nothing is refused, not stored. (A graph imported in the console is not checked this way.)

These spans are the whole of the traceability. CONCEPT → CODE opens them. CODE → CONCEPT (`TraceIndex`, `GET /api/projects/{id}/diagram-trace`) returns every node whose span covers a line, narrowest first.

## Provenance and confidence

- `observed`: the element is in the code as written (a branch, a call, a write).
- `inferred`: the session's reading of the code (a concept, a grouping of lines into one action).
- `documented`: taken from comments, docs or commit messages.
- `stated`: a person said so.

`confidence` is how sure the session is that the element is real and correctly placed. It is drawn as an arc. Leave it out rather than guess a number.

## Refusals

A refused graph comes back with one line per broken rule, each with its path (a 400 over REST, the tool's answer over MCP), for example:

```
The graph breaks 2 rule(s):
nodes[1].confidence: must be between 0 and 1
edges[0].to: no node has the id "ghost"
```

The spans are checked only once the graph itself passes, and report the same way (`nodes[4].sources[0].endLine: line 312 is past the end of src/order.rs (240 lines)`).

At most 50 rules are listed. Fix them all and send the whole graph again: a graph is written whole, never patched.

## Writing a good graph (guidance)

- Answer the question that was asked. About 10 to 40 nodes reads well on a screen, and past 80 a diagram stops being read.
- Break monolithic code into what it does. One `code` node for the function `contains` one node per validation, decision, state change, call and write inside it, each with its own line span.
- Every decision gets one `conditionally_leads_to` per outcome, each labelled with the outcome.
- Put data and external systems in as nodes, and connect them with `reads`, `writes` and `calls` rather than mentioning them in descriptions.
- Trace everything that has a place in the code. A concept with no single place is fine, and its description should say why.
- Use stable, readable ids (`validate-order`, not `n17`). A later update keeps the same ids, so the reader's picture survives.
- `title` says what the thing does. `description` says what the reader needs and the title cannot hold.

## Stored

A diagram belongs to a project, whose folder its paths are relative to. It optionally names the task it was generated from, and it keeps the question it answers. The graph is one JSON column (`diagram.graph_json`), written whole by `DiagramService::write` after validation. `POST`/`PUT /api/diagrams` call it directly. `rekall_diagram` goes through `DiagramService::write_generated`, which resolves the task's anchors, audits the spans against the folder, and stores the diagram on that task's project with the task as its origin. Passing `diagram` replaces that diagram in place, keeping its id and creation time.
