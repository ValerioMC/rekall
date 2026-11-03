## Explaining

Read `generating.md` too: this mode applies its rules.

`/rk project:vega explain "how an order is placed"` means: walk through the code that answers the
request as a **walkthrough**, a diagram the console plays one message at a time, showing the method
called, its code, what goes in and what comes back, and store it with `rekall_diagram`. A `task:`
anchor may come too, as in `generating.md`.

The text in double quotes is the request, in my words: the functionality I want to understand. It is
not an anchor. A walkthrough exists because a static diagram or a written explanation was not enough,
so it follows the code concretely, call by call.

1. Drop the `explain` term and the request, and call `rekall_context` with the anchors that are left.
   As in `generating.md`, you build nothing and no checklist changes.
2. Find the entry point the request starts from: the controller handler, the command, the scheduled
   job or the listener that is triggered first. Follow the calls from there through the layers, and
   read every method you are going to show.
3. Sheet 0 is the entry. It is a `sequenceDiagram` whose first message is the call into the entry
   point, from an `actor` or the external caller, and whose participants follow the call through the
   layers in order (controller, service, repository...). Every system outside the project's code (a
   database, an HTTP API, a queue, the filesystem) is a participant of its own, and its id goes in
   `walkthrough.external`.
4. Each call is a message labelled with the method and its arguments (`place(order)`), linked by `#n`
   to the method's body, with a dashed reply carrying what comes back. Give each call an entry in
   `walkthrough.messages` under the same `#n`: `explain` says what the step does and why, in one or
   two sentences drawn from the code; `input` what the method receives; `output` what it returns, or
   the error it raises. A reply needs an entry only when it says something its call does not.
5. A call whose own implementation is worth walking through gets a sequence sheet of its own, which
   starts at that call's receiver and whose title names the call. The call's entry points at it with
   `opens`, the sheet's index in `sheets`, counted from 0. The console opens it when the reader steps
   into the call and comes back when it ends. Nest as deep as the code needs, within 8 sheets.
6. Keep to what the code supports. `input` and `output` are values of the types the code declares: a
   short example when the code makes it concrete (`{ sku: "A-7", qty: 2 }`), the type's shape when it
   does not (`Order { lines: Vec<Line> }`). Never show a call, a value or a branch you did not see.
7. Call `rekall_diagram` with the anchors, a short `title`, the request as `question` and every sheet.
   If the answer is a refusal, fix every line it lists and send every sheet again.
8. Say in two or three lines what the walkthrough covers, how many messages and sheets it plays, and
   its id. Then stop: no wrapup, no step, no commit.
