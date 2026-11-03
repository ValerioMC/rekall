import type { TaskId } from '@/model/branded'

/** A task another one waits for, and whether it is accepted yet. */
export interface TaskDependency {
  readonly taskId: TaskId
  readonly anchor: string
  readonly title: string
  readonly met: boolean
  /** Claimed work only waits for the review that accepts it. */
  readonly awaitingReview: boolean
}

export function dependencyStateText(dependency: Pick<TaskDependency, 'met' | 'awaitingReview'>): string {
  if (dependency.met) return 'Accepted'
  return dependency.awaitingReview ? 'Claimed, waiting for your review' : 'Not accepted'
}
