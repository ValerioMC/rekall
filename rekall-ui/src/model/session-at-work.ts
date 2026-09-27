import type { Task, TaskStep } from '@/model/catalog'
import type { Terminal } from '@/model/terminal'
import type { TaskStepId } from '@/model/branded'

/**
 * Whether a session is still working what a "Run here" button launches, read from what the server
 * already reports live. A step is worked while it is `RUNNING`: the server marks it when its
 * terminal opens and moves it on a claim or when the terminal ends. A stepless task follows its
 * own review line the same way. A checklist task launched as a whole is worked while one of its
 * steps runs, or while its terminal is live and a step is still open.
 */
export function sessionAtWork(
  task: Pick<Task, 'id' | 'reviewActive' | 'reviewState'> | null,
  steps: readonly Pick<TaskStep, 'id' | 'taskId' | 'state'>[],
  terminals: readonly Pick<Terminal, 'taskId' | 'live'>[],
  stepId: TaskStepId | null
): boolean {
  if (!task) return false
  if (stepId) return steps.some((step) => step.id === stepId && step.state === 'RUNNING')
  if (task.reviewActive) return task.reviewState === 'RUNNING'
  const own = steps.filter((step) => step.taskId === task.id)
  if (own.some((step) => step.state === 'RUNNING')) return true
  const live = terminals.some((terminal) => terminal.taskId === task.id && terminal.live)
  return live && own.some((step) => step.state === 'OPEN')
}
