## Generating

`/rk project:vega generate kinds:sequence,state "how checkout decides what to charge"` means: work
out what the code does to answer that request, describe it as Mermaid sheets whose elements are
linked to the code, and store them with `rekall_diagram`. The console draws them; you never do.
A `task:` anchor may come too (`/rk project:vega task:report-builder generate ...`): the diagram is
then stored on that task as well as on its project.

The text in double quotes is the request, in my words: what the diagram has to make clear. It is not
an anchor. `kinds:` lists the Mermaid types I asked for, one sheet each, in that order; it is not an
anchor either.

1. Drop the `generate` term, the `kinds:` term and the request, and call `rekall_context` with the
   anchors that are left. The description, wrapup and notes tell you what the project is and where
   things live. They are not work: in this mode you build nothing, and no checklist changes.
2. Find the code yourself. Your working directory is the project's folder, and every path you link
   is relative to it. Do not ask me where the project is, which files to read, which functions
   matter or where to save: search, read, follow calls and references until you can answer the
   request, and read only what it needs.
3. Write one sheet per type in `kinds:`. Without `kinds:`, choose what answers the request: a
   sequence for who calls whom and in what order, a state diagram for a lifecycle, a flowchart for
   the decisions inside one piece of code, a class or ER diagram for structure.
4. Describe the behaviour, not the file layout. In a sequence the participants are the components
   that talk (types, modules, services, external systems), each message is a call labelled with
   what is called (`charge(card)`), its dashed reply carries what comes back, `+`/`-` activations
   nest the calls the way the code does, and branches and repetition go in `alt`, `opt` and `loop`
   with their condition. A 200-line function that validates, charges, stores and notifies is four
   messages or four nodes, each linked to its own lines. A status field set on different branches
   is a state diagram even when the code never calls it one.
5. Keep to what the code supports. Link every participant, message, node, state, class or entity
   that has a place in the code to that place in `links`, and read those lines before you cite them:
   a participant to its type, a message to the body of the method it calls. Never draw a call, a
   transition or a write you did not see.
6. Call `rekall_diagram` with the anchors, a short `title`, the request as `question` and every
   sheet. Its description carries the full format: follow it, and answer the request rather than the
   whole codebase. If the answer is a refusal, fix every line it lists and send every sheet again.
7. Say in two or three lines what the diagram shows, how many elements are linked to code, and its
   id. Then stop: no wrapup, no step, no commit. A diagram is not the task's work.

The sheets are the whole output. Never HTML, SVG, Vue or styling, and no rendition of the diagram in
the chat either. To revise a diagram whose id you have, send every sheet again with that id as
`diagram`, keeping the ids of the elements that stay.
