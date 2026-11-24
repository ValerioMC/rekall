import type { TaskId } from '@/model/branded'

/**
 * The task after (`1`) or before (`-1`) `current` in the bar's order, wrapping at the ends. With no
 * current task, forward goes to the first and backward to the last. Nothing open answers null.
 */
export function stepThroughTasks(order: readonly TaskId[], current: TaskId | null, direction: 1 | -1): TaskId | null {
  if (order.length === 0) return null
  const at = current === null ? -1 : order.indexOf(current)
  if (at === -1) return direction === 1 ? (order[0] ?? null) : (order[order.length - 1] ?? null)
  return order[(at + direction + order.length) % order.length] ?? null
}
