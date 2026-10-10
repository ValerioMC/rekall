# Diagrams

The format a session uses to answer a question about what a piece of software does. A diagram is
one or more **Mermaid** sheets, and each sheet can tie the elements it draws to the code that
implements them. Rekall stores the document as it arrives and draws it with its own code. A session
never produces HTML, SVG, Vue or styling, only this document.

The types, the reader and the rules live in `rekall-diagram` (`DiagramDocument`, `DocumentCodec`,
`DocumentValidator`, `MermaidText`). Every rule below is one the validator enforces, unless it is
marked as guidance.

## The document

```json
{
  "sheets": [
    {
      "title": "Checkout",
      "mermaid": "sequenceDiagram\n  participant S as OrderService\n  participant P as Payments\n  S->>+P: charge(card)\n  P-->>-S: receipt",
      "links": {
        "S": [{ "file": "src/order.rs", "startLine": 10, "endLine": 120, "symbol": "OrderService" }],
        "#1": [{ "file": "src/payment.rs", "startLine": 42, "endLine": 61, "symbol": "Payments::charge" }]
      }
    }
  ]
}
```

- A diagram holds between 1 and 8 sheets, shown as tabs in the order given.
- No other fields are accepted, at any level. A field the format does not define is refused rather
  than dropped, so a typo cannot pass silently.

| Field | Required | Meaning |
|---|---|---|
| `mermaid` | yes | The Mermaid source, at most 100 000 characters. Its first line declares the type |
| `title` | no | The sheet's tab label, at most 120 characters. Without one the tab shows the type |
| `links` | no | Which elements open which code. See Links |

## Types

The first line of `mermaid` (after any front matter, `%%` comments and `%%{init}%%` directives)
declares the type. Anything else is refused.

| Type | Declared with | Drawn by | Links address |
|---|---|---|---|
| `sequence` | `sequenceDiagram` | Rekall | participants by id, messages by `#n` |
| `flowchart` | `flowchart TD`, `flowchart LR`, `graph …` | Rekall | nodes by id |
| `state` | `stateDiagram-v2`, `stateDiagram` | Rekall | states by id |
| `class` | `classDiagram` | Rekall | classes by name |
| `er` | `erDiagram` | Rekall | entities by name |
| the rest | `gantt`, `pie`, `mindmap`, `timeline`, `journey`, `quadrantChart`, `gitGraph`, `requirementDiagram`, `C4Context` and the other C4 types, `sankey-beta`, `xychart-beta`, `block-beta`, `packet-beta`, `architecture-beta`, `kanban`, `radar-beta` | Mermaid | nothing: a sheet of these types carries no `links` |

Rekall draws the first five with its own components and decides how they look, so styling
statements (`style`, `classDef`, `linkStyle`, `click`, themes) change nothing there. Of a sequence
it draws participants and actors, participant boxes, every arrow type, activations (`+`/`-` and
`activate`/`deactivate`), notes, `loop`, `alt`/`else`, `opt`, `par`/`and`, `critical`/`option`,
`break` and `rect` blocks, and `autonumber`. Of a flowchart, every node shape, edge labels, dotted
and thick edges, and subgraphs. Of a state diagram, start and end, choice, fork and join, composite
states and transition labels. Of a class diagram, members, methods, annotations and every relation
with its cardinalities. Of an ER diagram, attributes with their keys and every relationship's
cardinality.

## Links

`links` maps an element of the sheet to where it lives in the code:

- A key is an element's Mermaid **id**: the participant id in `participant S as OrderService` is
  `S`, the node id in `validate[Validate order]` is `validate`, a class or an entity is its name.
  The id has to appear in the sheet as a whole word.
- In a sequence, `#n` addresses the n-th **message**, counting every arrow from the top in order,
  the number `autonumber` prints. Notes, blocks and activations are not messages. `#n` past the
  last message is refused, and `#n` on any other type is refused.
- Each key lists 1 to 8 sources. A sheet carries at most 400 keys.

A source:

```json
{ "file": "src/order.rs", "startLine": 42, "endLine": 61, "symbol": "process_order" }
```

- `file` is required and not blank. It is relative to the project's folder (`./` and `\` are
  normalised when compared). The console reads code only inside that folder, and a path that
  leaves it is refused.
- Lines are one-based and inclusive. `startLine` 0 is refused. An `endLine` needs a `startLine`
  and cannot come before it. Without lines, the link claims the whole file.
- `symbol` names the enclosing function, method or type when there is one.
- A diagram stored through `rekall_diagram` has every span held against the project's folder
  before it is written: the file has to be there and `startLine` and `endLine` inside it. A span
  that points at nothing is refused, not stored. (A diagram imported in the console is not checked
  this way.)

In the console, clicking a linked element opens its code. In a sequence it also shows the
**call tree** under it, read from the messages:

1. A solid arrow (`->>`, `->`, `-x`, `-)`) is a call from its sender to its receiver.
2. A call nests under the call its sender is currently answering, so a chain of calls becomes a
   tree. A call a participant makes to itself is a leaf: nothing nests under it.
3. A dashed arrow (`-->>`, `-->`, `--x`, `--)`) is a reply. It closes the call it answers and every
   call opened inside it.

## Writing a good diagram (guidance)

- Answer the question that was asked, with one sheet for each type requested. About 6 to 12
  participants or 10 to 40 nodes reads well on a screen.
- In a sequence, participants are the components that talk: a type, a module, a service, an
  external system. Declare each one with `participant Id as Label` (or `actor` for a person) and
  link it to the type or module that implements it.
- A message is a call, labelled with what is called and with what (`charge(card)`). Link each one
  to the body of the method it calls. Give every call its reply (`-->>`) carrying what comes back,
  and use `+`/`-` activations, so the call tree nests the way the code does.
- Put branches in `alt`/`else` with the condition as the label, repetition in `loop`, and what
  happens only sometimes in `opt`.
- Break monolithic code into what it does. A 200-line function that validates, charges, stores and
  notifies is four messages or four nodes, each linked to its own line span.
- Use stable, readable ids (`orders`, `validate`), not `A`, `B`, `n17`. A revision keeps the same
  ids, so the reader's picture survives.

## Refusals

A refused document comes back with one line per broken rule, each with its path (a 400 over REST,
the tool's answer over MCP), for example:

```
The diagram breaks 2 rule(s):
sheets[0].links["#7"]: there is no message #7: the sequence sends 5
sheets[1].mermaid: the first line has to declare a Mermaid diagram type, such as `sequenceDiagram`, …
```

The spans are checked only once the document itself passes, and report the same way
(`sheets[0].links["S"][0].endLine: line 312 is past the end of src/order.rs (240 lines)`).

At most 50 rules are listed. Fix them all and send the whole document again: a diagram is written
whole, never patched.

## Stored

A diagram belongs to a project, whose folder its paths are relative to, and optionally names the
task it was generated for. It keeps the question it answers. The document is one JSON column
(`diagram.document_json`) beside the sheets' types (`diagram.kinds`), written whole by
`DiagramService::write` after validation. `POST`/`PUT /api/diagrams` call it directly.
`rekall_diagram` goes through `DiagramService::write_generated`, which resolves the anchors (a
project, or a task and its project), audits the spans against the folder, and stores the diagram.
Passing `diagram` replaces that diagram in place, keeping its id and creation time.
