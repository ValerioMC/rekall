## Merging

`/rk task:RK-12 merge` means: the task is accepted and its branch conflicts with the
project's current branch. Bring the project's branch into the task's branch, in the working directory,
with a merge commit (never a rebase), and resolve what conflicts. Rekall merges the task's branch into the project itself once this branch
merges cleanly, then closes this session and removes the worktree.

1. Drop the `merge` term and call `rekall_context` with the anchors that are left. You change no
   checklist, no wrapup and no task: do not call `rekall_wrapup`, `rekall_step` or `rekall_record_commit`.
2. Check the working directory is the task's worktree, on its own `rekall/…` branch
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
decision is, and stop. Never rebase, never push, never force, never delete a branch, never switch the working
directory to another branch.

### When the terms include `pull-request`

`/rk task:RK-12 merge pull-request base:main` means: the task is accepted, its
pull request is open, and the host says it conflicts with `base`, the branch it targets. Bring `base`
into the task's branch, in the working directory, with a merge commit (never a rebase), resolve what
conflicts, and push the branch. Rekall merges the pull request through the host itself once it merges
cleanly, then closes this session and removes the worktree. Everything above applies except the
instructions about which branch to merge and about pushing:

1. Drop the `merge`, `pull-request` and `base:` terms and call `rekall_context` with the anchors that
   are left. You change no checklist, no wrapup and no task.
2. Check the working directory is the task's worktree, on its own `rekall/…` branch. If it is
   not, say so and stop.
3. `git fetch origin`, then merge the base into this branch with `git merge --no-edit origin/<base>`.
4. Resolve each conflicted file, run the project's build and tests and commit the merge, as above.
5. `git push origin <that branch>`. If the remote refuses because the branch moved, fetch, merge
   `origin/<base>` again and push again.
6. Say in at most five lines what conflicted, how you resolved each file, and what you ran.

Never force-push (no `--force`, no `--force-with-lease`, no `+` refspec). Never merge or close the pull
request and never call the host's merge API: Rekall does that when the push is in. If a conflict needs a
decision that is mine, run `git merge --abort`, say what it is, and stop.
