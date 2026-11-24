import type { ReviewItem } from '@/features/catalog/model/review'
import type { TaskId } from '@/model/branded'

/** How many pieces of work wait for review on each task. A task with none has no entry. */
export function reviewCountsByTask(queue: readonly ReviewItem[]): ReadonlyMap<TaskId, number> {
  const counts = new Map<TaskId, number>()
  for (const item of queue) counts.set(item.taskId, (counts.get(item.taskId) ?? 0) + 1)
  return counts
}
