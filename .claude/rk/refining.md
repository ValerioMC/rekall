## Refining

`/rk task:RK-12 refine` means: go through this task with me before anything runs, and record what we
settle. It is talk and records. It changes no code, no steps, no description and no wrapup.

1. Drop the `refine` term and call `rekall_context` with the anchors that are left. Read the
   description, the steps, the wrapup and the `<decisions>` block: a record in it is already settled
   or already asked, and is never asked again.
2. Find what is missing or could be better for the task to run without a question mid-way: choices
   that change the manifests, the schema or the plan, things the description assumes, steps that
   depend on an answer nobody has given. Ask one question at a time, in the terminal, in my words:
   what it is, why it matters, and the options you see with what each one causes and the one you
   recommend.
3. Record each question as you ask it, with `rekall_decision` `ask`: a `key` (a lowercase slug), the
   `question`, a `topic` that groups it, `why`, the `options`, the `step` number when it concerns one
   step and not the whole task, and the `lines` of the conversation behind it. A task without steps
   takes no step number.
4. When I answer, record it with `answer`, and the `lines` that settled it. Record only what I
   settled: never a guess, a "maybe" or an answer you inferred. If I say I am finding something out,
   `hold` it with a one-line note of what I am checking and go on. Later answers end the hold.
5. When I change my mind or add to an answer, use `correct`: a new answer keeps the old one, and a
   `detail` is kept beside the answer with its date. A record that was wrong from the start is
   `remove`d. A record marked `[edited by you]` was written by me in the console: do not contradict
   it, ask me if it no longer fits.
6. Stop when I say it is enough. Say in a few lines how many decisions the task carries, how many
   are open and how many are on hold, and anything you did not get to. Build nothing.
