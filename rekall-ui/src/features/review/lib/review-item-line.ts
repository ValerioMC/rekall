import type { Task } from '@/features/catalog/model/catalog'

export interface StepPlace {
  readonly number: number
  readonly total: number
}

/** What a claim says under its title: where the work is, company first, and which piece of it waits. */
export function reviewItemLine(task: Task, place: StepPlace | null): string {
  const piece = place === null ? 'whole task' : `step ${place.number} of ${place.total}`
  return `${task.companyName} · ${task.projectTitle} · ${piece}`
}
