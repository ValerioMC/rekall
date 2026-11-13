import type { Task } from '@/features/catalog/model/catalog'

/** What a queue row says under its title: where the work is, company first, and which piece of it waits. */
export function reviewItemLine(task: Task, stepNumber: number | null): string {
  const piece = stepNumber === null ? 'whole task' : `step ${stepNumber}`
  return `${task.companyName} · ${task.projectTitle} · ${piece}`
}
