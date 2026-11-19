import type { Task } from '@/features/catalog/model/catalog'

export function taskDeletionBody(trackerIssueKey: string | null): string {
  const notes = 'The notes on it are deleted with it.'
  if (!trackerIssueKey) return notes
  return `${notes} Issue ${trackerIssueKey} is deleted from the project's task manager too.`
}

export function taskDeletionBlast(task: Pick<Task, 'documentCount' | 'trackerIssueKey'> | undefined): string {
  const count = task?.documentCount ?? 0
  const notes = `${count} note${count === 1 ? '' : 's'} deleted`
  return task?.trackerIssueKey ? `${notes} · issue ${task.trackerIssueKey} deleted` : notes
}
