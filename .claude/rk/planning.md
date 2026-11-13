## Planning

`/rk task:RK-12 plan` means: turn the task into a checklist, ready to work.

1. Drop the `plan` term, and `flow` after it, and call `rekall_context` with the anchors that are left.
2. Read what it gives you: the description says what the work is for and what it has to satisfy,
   the wrapup what already exists, the done steps what is finished. Then read the code the task
   touches, enough to know where each piece will go.
3. Draw the solution wherever a picture says more than words (see **Drawing the plan** below),
   before writing the steps, so each step is cut from a design and not from the description alone.
4. Decide whether the work is a checklist or a flow. It is a flow when a step's outcome decides
   what comes next: tests that can fail and send the work to a fix, a deploy that can break, a
   review that can reject. Work that only goes from one step to the next is a checklist, however
   long. The bare word `flow` after `plan` (`/rk ... plan flow`) asks for a flow outright.
5. For a checklist, call `rekall_propose_step` once per step, in the order the work should be done.
   For a flow, call `rekall_propose_flow` once with every step in order, the first where it begins,
   and each step's routes: `{"outcome": "app-error", "to": 3}`, or `"to": "end"`. Do not pass
   `draft`: the steps land open and ready to work, promoted on their own. Pass `draft: true` only
   when I ask for drafts to review.
   Work out the arcs one step at a time: what can end it (it worked, and each way it can fail) and
   where each of those goes (the next step, back to a fix step, or the end). Name outcomes for
   what happened (`passed`, `app-error`), give a step that simply leads on one route to the next,
   leave the steps that finish the work without routes, and give every loop a way out. Give the
   step a loop returns to a `max_visits` (3 is a sensible start), so the flow stops instead of
   going round forever. Either way, a
   title says what the step delivers; the detail says what to build, where, what it must satisfy
   and how I can tell it is done, enough for a session that never saw this one to do it alone.
   Propose only what is still missing, and nothing the description rules out.
   Cut the sessions as part of the plan, as **Splitting into sessions** says: never wait to be asked.
   Close the plan as **Testing** says.
6. Stop. Say in a few lines how many steps (and, for a flow, routes) you proposed, how many
   sessions they make and where each starts, which diagrams you drew and where they are, and
   anything you left out on purpose. Build nothing: the steps are
   open, and I review them in the console if I want to.

A title the task already has is refused, so a second `plan` adds only what the first one missed. A
flow is proposed onto a task with no steps, or whose steps are all claimed or done: those stay as
history and the flow begins at your first new step. A route's `to` counts the whole checklist as the
console lists it, so after four finished steps the first new step is step 5; a route to a step that
stays is refused. Design the routes yourself. If the tool refuses, quote the refusal and name the way
out: a step still `running` is claimed first, a title already taken is renamed.

### Splitting into sessions

A plan is also a plan of sessions. Every turn of a session re-reads everything the session has seen,
so a long one costs more and reasons worse on work unlike what it holds. Decide where sessions
start whenever you plan or replan, without being told to, and mark each start with `new_session`
(`true`) on the step, in `rekall_propose_step`, `rekall_propose_flow` or `rekall_replan`.

- Keep steps together when they work the same area and each builds on what the one before read.
- Open a new session when the area changes (another crate, the backend then the console, code then
  docs), when earlier steps leave a lot to unlearn (a refactor that moved the code around), when a
  step is big enough to need its own room, or when a session would otherwise hold more than a few
  large steps.
- A step that only checks or documents what the steps before it built stays in their session.
- On a flow, group by the order the steps are worked, not the order they are listed; a step a loop
  returns to does not open a session, because it needs what the failed pass left.
- On a replan, steps that stay keep their sessions. The first new step joins the last kept step's
  session unless it meets one of the reasons above.
- A checklist of a few small steps stays one session: the first step opens it, and no other step
  carries `new_session`. Do not split for the sake of splitting.

The wrapup written before a session's last claim is what the next session starts from, so a split
costs nothing the wrapup does not carry. Say in the closing lines of the plan how many sessions it
has and where they start.

### Testing

A step is tested by what it built, and the whole suite runs once, at the end.

- Say in each step's detail which tests it adds or changes and how to run only those (one Rust test
  or crate, one spec file). A step never asks for the project's whole test, typecheck and lint run.
- The last step of the plan is a dedicated verification step: it runs the project's whole suite
  (build, tests, typecheck, lint, as its sections or `CLAUDE.md` say) and fixes what fails. It stays
  in the session of the steps before it.
- On a flow, that step carries the routes: `passed` -> end of the flow, and a route per kind of
  failure back to the step that fixes it, with a `max_visits` on the step it returns to. A step
  that fixes a failure runs only the tests of what it fixed, then goes back to verification.
- A task whose work is not code (documentation, a note, a prompt) has no verification step.

### Drawing the plan

A description is often too broad to build from, and in a long plan the later steps are the vaguest.
A diagram fixes the shape of the solution before the steps are cut from it. Draw one **when it makes
the solution clearer than the words do**; a copy change or a fix in one function gets none, and a
diagram that repeats its step's sentence is noise.

- Pick the kind by what the solution is: `sequenceDiagram` for who calls whom and in what order,
  `erDiagram` for stored data, `classDiagram` for types and how they relate, `stateDiagram-v2` for a
  lifecycle, `flowchart` for a decision. Name the elements as the code names them, and draw only what
  the solution has or will have. The solution is usually several layers (a table, a repository, a
  service, an endpoint, a screen): show each layer's piece, not one box for all of them.
- **The whole solution** goes in a note on the task, written once with `rekall_note` as
  `solution-overview.md`, when the plan has more than a handful of steps or crosses layers. The
  details of the steps it covers name the note.
- **Each step's own piece** goes in the step's `detail`, as a fenced `mermaid` block, drawn in its
  place among its neighbours: the parts that already exist and the ones other steps build, the new
  ones marked. A step in the middle of a long plan then shows what it builds and what it touches
  without sending the session to the description. A step whose piece is one function gets no block.
- A diagram already written in the description or in a note is the starting point: take it, correct
  what the code shows is wrong and carry on from it, rather than drawing a second one beside it.
- The console draws a block that parses and shows an error for one that does not, so keep to plain
  Mermaid, quote labels with punctuation, and give no sequence participant an id that starts with
  `x`: the lexer reads `-X` after a deactivation as the cross arrow.
