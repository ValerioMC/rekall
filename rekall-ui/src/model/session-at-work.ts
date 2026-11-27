import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { Terminal } from '@/features/claude/model/terminal'

/** Work is on while the review line or a step is `RUNNING`, or a live session that is no discussion has an open step left. */
export function sessionAtWork(
  task: Pick<Task, 'id' | 'reviewActive' | 'reviewState'> | null,
  steps: readonly Pick<TaskStep, 'taskId' | 'state'>[],
  terminals: readonly Pick<Terminal, 'taskId' | 'live' | 'mode'>[]
): boolean {
  if (!task) return false
  if (task.reviewActive) return task.reviewState === 'RUNNING'
  const own = steps.filter((step) => step.taskId === task.id)
  if (own.some((step) => step.state === 'RUNNING')) return true
  const working = terminals.some((terminal) => terminal.taskId === task.id && terminal.live && terminal.mode !== 'DISCUSS')
  return working && own.some((step) => step.state === 'OPEN')
}
