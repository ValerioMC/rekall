import type { Task } from '@/features/catalog/model/catalog'
import { newestFirst } from '@/features/catalog/model/task-code'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskId } from '@/model/branded'
import type { TokensOfTask, WorkKind, WorkRow } from '@/features/work/model/work-view'

function claimedStepsByTask(steps: readonly TaskStep[]): Map<TaskId, number> {
  const claimed = new Map<TaskId, number>()
  for (const step of steps) {
    if (step.state === 'CLAIMED') claimed.set(step.taskId, (claimed.get(step.taskId) ?? 0) + 1)
  }
  return claimed
}

function runningTaskIds(tasks: readonly Task[], steps: readonly TaskStep[]): Set<TaskId> {
  const running = new Set<TaskId>()
  for (const step of steps) {
    if (step.state === 'RUNNING') running.add(step.taskId)
  }
  for (const task of tasks) {
    if (task.reviewActive && task.stepCount === 0 && task.reviewState === 'RUNNING') running.add(task.id)
  }
  return running
}

/** A task handed in whole: it has no checklist and its wrapup claimed it. The review queue counts it too. */
export function handedInWhole(task: Task): boolean {
  return task.reviewActive && task.reviewState === 'CLAIMED'
}

function kindOf(task: Task, running: boolean, needs: number): WorkKind {
  if (task.status === 'DONE') return 'filed'
  if (running) return 'running'
  if (needs > 0) return 'review'
  if (task.status === 'TODO') return 'todo'
  if (task.status === 'BLOCKED') return 'blocked'
  if (task.status === 'BACKLOG') return 'backlog'
  return 'progress'
}

/** One row per task, newest first. `steps` may hold every task's steps. */
export function workRows(tasks: readonly Task[], steps: readonly TaskStep[], tokensOf: TokensOfTask): WorkRow[] {
  const claimed = claimedStepsByTask(steps)
  const running = runningTaskIds(tasks, steps)
  return [...tasks].sort(newestFirst).map((task): WorkRow => {
    const claimedSteps = claimed.get(task.id) ?? 0
    const needs = claimedSteps + (handedInWhole(task) ? 1 : 0)
    const isRunning = running.has(task.id)
    return { task, kind: kindOf(task, isRunning, needs), claimedSteps, needs, running: isRunning, tokens: tokensOf(task.id) }
  })
}
