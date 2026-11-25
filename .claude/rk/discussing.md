## Discussing

`/rk task:RK-12 discuss` means: talk this task over with me, because I want to clarify something. I
lead the discussion. It is a clarification only: the task is never executed.

1. Drop the `discuss` term and call `rekall_context` with the anchors that are left.
2. Gather the rest: `rekall_read_steps` for the draft and open steps as stored, every note that came
   on request by its `note:` anchor, and the project pages the talk touches. Read code only to answer
   me or to word a change.
3. Say in a few lines what is still to do, then ask what I want to clarify and let me lead. Do not
   walk the task looking for gaps, do not question me down a list and do not propose how to build it:
   that is `refine`. Ask only what is needed to understand my point; say how I could do it when I ask.
4. Before writing, say exactly what will change and apply it only once I agree. This repeats as the
   talk goes on.
5. Write each change where it belongs:
   - the description: `rekall_description`, with the whole text and nothing dropped. Write `[[...]]`
     references back as typed (`[[Name]]`, `[[connection:label]]`, `[[task:RK-12]]`), not as the
     context expanded them.
   - a settled answer: `rekall_decision`. Never contradict a record marked `[edited by you]` without
     asking me.
   - a note: `rekall_note`, with `replace` for an existing one.
   - a step's title or detail: `rekall_edit_step`.
   - a new step: `rekall_propose_step`, or `rekall_propose_flow` on a task with no steps.
   - a dropped step: `rekall_delete_step`.
   - a new order or structure: `rekall_replan`, by the rules of `planning.md`. Say first that it
     deletes every draft and open step with its skills, checks and routes.
6. The scope is the work still to do. Running, claimed and done steps are never changed, reopened or
   re-decided, and description text about built work stays as written. New work is a new step; on a
   task already run it must be a step, since a description written here opens no revision.
7. Never edit code or files, never call `rekall_step`, `rekall_wrapup`, `rekall_feedback` or
   `rekall_deviation`, and never commit.
8. Stop when I say it is enough and list what changed. Build nothing.
