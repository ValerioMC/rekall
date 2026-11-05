## Merging

`/rk project:vega task:report-builder merge` means: the task is accepted and its branch conflicts with the
project's current branch. Bring the project's branch into the task's branch, in the working directory,
and resolve what conflicts. Rekall merges the task's branch into the project itself once this branch
merges cleanly, then closes this session and removes the worktree.

1. Drop the `merge` term and call `rekall_context` with the anchors that are left. You change no
   checklist, no wrapup and no task: do not call `rekall_wrapup`, `rekall_step` or `rekall_record_commit`.
2. Check the working directory is the task's worktree on the branch `rekall/<task-label>`
   (`git branch --show-current`). If it is not, say so and stop: nothing here may touch the project's
   own checkout.
3. Find the branch checked out in the project's folder: the first entry of `git worktree list --porcelain`.
   Merge it into this branch with `git merge --no-edit <that branch>`.
4. For each conflicted file, read both sides and the task's description and wrapup, then write the
   result that keeps the intent of both. Never drop one side to make the conflict go away, and never
   use `-X ours` or `-X theirs` for the whole merge.
5. Run the project's build and tests, as its sections say. Fix what the merge broke. A failure that
   was already there before the merge is reported, not fixed.
6. `git add` the resolved files and finish with `git commit --no-edit`. The worktree must be clean.
7. Say in at most five lines what conflicted, how you resolved each file, and what you ran.

If a conflict cannot be resolved without a decision that is mine, run `git merge --abort`, say what the
decision is, and stop. Never push, never force, never delete a branch, never switch the working
directory to another branch.
