import type { TaskBranch } from '@/features/worktree/model/taskBranch'

export function mergeText(branch: Pick<TaskBranch, 'mergesCleanly'>): string {
  if (branch.mergesCleanly === null) return 'Git could not tell whether it merges cleanly.'
  return branch.mergesCleanly ? 'Merges cleanly into the project\'s current branch.' : 'Conflicts with the project\'s current branch.'
}

export function aheadText(branch: Pick<TaskBranch, 'commitsAhead' | 'changedFiles'>): string {
  const commits = `${branch.commitsAhead} ${branch.commitsAhead === 1 ? 'commit' : 'commits'} ahead`
  const files = `${branch.changedFiles.length} ${branch.changedFiles.length === 1 ? 'file' : 'files'} changed`
  return `${commits}, ${files}`
}

export function worktreeText(branch: Pick<TaskBranch, 'worktreeFolder' | 'hasUnsavedChanges'>): string {
  if (branch.worktreeFolder === null) return 'No worktree: its folder was removed, the branch and its commits stay.'
  return branch.hasUnsavedChanges
    ? 'The worktree has unsaved changes, so Rekall leaves it where it is.'
    : 'The worktree is clean. It is removed when the task is accepted.'
}
