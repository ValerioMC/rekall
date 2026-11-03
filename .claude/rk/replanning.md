## Replanning

Read `planning.md` too: this mode applies its rules.

`/rk project:vega task:report-builder replan` means: redesign the steps this task already has.
`/rk project:vega task:report-builder replan "also cover the csv export"` means the same, and the
quoted text is what I noticed the plan missed, in my words. It is not an anchor.

1. Drop the `replan` term and the quoted text, and call `rekall_context` with the anchors that are
   left. The anchors string never carries the quoted text.
2. Read what it gives you as in `planning.md`, and read the steps that are there: which are done,
   which are running or claimed, which are open or drafts. Work out what is wrong with the open
   and draft ones, and what still has to be built. With quoted text, that is what to find: read it
   as the new requirement, change the open and draft steps it touches, keep the ones it does not,
   and add the steps it needs.
3. Decide checklist or flow and work out the arcs as in `planning.md`. Steps that are running,
   claimed or done stay and are not planned again, so the new steps follow them: when any stay,
   plan a checklist and say the routes are mine to draw. Draw the new steps as in **Drawing the
   plan** in `planning.md`, starting from the diagrams the task already carries and correcting them; a note that
   already holds the overview is rewritten with `replace: true`. Cut the new steps into sessions as
   **Splitting into sessions** in `planning.md` says, without being asked.
4. Call `rekall_replan` once with every new step. It deletes the task's draft and open steps and
   lays the new ones in a single transaction, and refuses the whole plan when any part is wrong.
   Leave `draft` out, as in `planning.md`. To drop a single step that belongs on no plan, call
   `rekall_delete_step`.
5. Stop. Say in a few lines how many steps were deleted, how many were laid, how many sessions the
   new steps make and where each starts, which steps stayed and anything you left out on purpose. Build nothing.
