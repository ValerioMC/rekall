## Stepping

`/rk project:vega task:report-builder step:3 start` and `/rk project:vega task:report-builder step:3
done` are how a session drives its own checklist as it works. `step:` takes the step's number as
`rekall_context` lists them, or its exact title.

- `start` marks that step **running**: call `rekall_step` with `state` `running`.
- `done` marks it **claimed**: call `rekall_step` with `state` `claimed`. It does not tick the box.
  Only I do that, in the console, once I have looked at the work.
- `outcome:<name>` alongside `done`, on a flow, is what happened: pass it as `outcome`. Rekall routes
  by it to the next step (see **The flow**).
- `report:"..."` alongside `done`, on a flow, is what you saw there: pass it as `report`. Give one with
  every failure.

A `<thread>` with `PENDING` or `RUNNING` entries is worked only through `revise` or `replan`, never while you work
a checklist step.

The loop, once you are working a checklist:

1. `/rk project:x task:y step:N start`, before you touch the code for step N.
2. Do the work the step's detail describes, and run only the tests of what it built or changed. The
   whole suite, typecheck and lint belong to the plan's final verification step, not to every step.
3. `/rk project:x task:y wrapup`, to fold what that step built into the task's wrapup, by the rules
   in `wrapping-up.md`. One state, rewritten whole, never a section per step. On a flow, skip this
   until the flow is finished.
4. `/rk project:x task:y step:N done`, to claim it.
5. If a later step is still open, `/rk project:x task:y step:M start` and go again, unless it opens a
   new session (see **Sessions**). When none is, stop and tell me the checklist is claimed and waiting
   for my review. On a flow, the next step is the one the claim's answer names and you start it without
   stopping; the loop ends when the answer says the flow is finished, and only then do you write the
   wrapup.

**Sessions.** A step that reads `(opens a new session)` is the first of a session of its own, and the `<steps>` tag
carries `sessions="N"` when there is more than one. A step without it belongs to the session the step
before it ran in. A checklist with no such step is one session from the first step to the last.

The wrapup is what the next session starts from, so write it before you claim the last step of a
session. When the step you claimed is followed by one that opens a new session, do not start that one
here, on a flow too: say which step the next session opens on and stop. Rekall closes this terminal and
opens a new one on that step. The `step:M start` of that next step is the new session's to make.

`rekall_step` refuses `done` as a state, and refuses a step I have already accepted. If it comes
back with either, say so and stop.

If the claim is refused for a check, correct the code until it passes and claim again. Do not
change the check.

If the project's context carries an `auto-commit` field, the claim is what commits: `step:N done`
(and, on a task with no checklist, the wrapup) stages and commits the folder and logs the commit
itself. Do not `git commit` on such a project; read the answer for what it committed, or why not.

On such a project, pass `commit_message` with the claim (with `rekall_wrapup` on a task with no
checklist), because it is the commit's message:

- The first line is a Conventional Commits subject under 72 characters, `feat: …` or `fix: …`, saying
  what the work delivers, not the step's number.
- Then a blank line and two to five sentences of plain prose: what the change does, why, and anything
  a reader of `git log` should know. No list of files, no "this session", no anchors: Rekall adds a
  `Refs:` line with the task and step itself.

Without it, Rekall derives the message from the step's title and detail, which is the fallback and
reads like one.
