import type { Project, Task } from './catalog'
import type { TaskDraft } from './record-draft'
import type { ProjectId } from '../../../model/branded'

/** The number a task's code ends with: the task's place in its project, 0 when the code has none. */
export function codeNumber(code: string): number {
  const number = Number(code.slice(code.lastIndexOf('-') + 1))
  return Number.isInteger(number) ? number : 0
}

/** Newest first: the higher a task's number, the earlier it is listed. */
export function newestFirst(a: Pick<Task, 'code'>, b: Pick<Task, 'code'>): number {
  return codeNumber(b.code) - codeNumber(a.code)
}

function nextNumber(projectId: ProjectId, tasks: readonly Task[]): number {
  const numbers = tasks.filter((task) => task.projectId === projectId).map((task) => codeNumber(task.code))
  return Math.max(0, ...numbers) + 1
}

/** The code the task has, or the one it will get: the next of the project it is filed in. */
export function taskCodeOf(draft: TaskDraft, projects: readonly Project[], tasks: readonly Task[]): string {
  const stored = tasks.find((task) => task.id === draft.id)
  if (stored && stored.projectId === draft.projectId) return stored.code
  const project = projects.find((candidate) => candidate.id === draft.projectId)
  return project ? `${project.taskKey}-${nextNumber(project.id, tasks)}` : ''
}

/** Whether a `[[task:...]]` reference names the task: by its code, or by the label older text still carries. */
export function namesTask(task: Pick<Task, 'code' | 'label'>, reference: string): boolean {
  const wanted = reference.trim().toLowerCase()
  return task.code.toLowerCase() === wanted || task.label.toLowerCase() === wanted
}
