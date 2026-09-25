import type { Task, TaskStep } from '@/model/catalog'

/**
 * What the mark in front of a task row says, beyond the task's status.
 *
 *   LIVE      a session or a timer is on the task right now, whatever its status.
 *   LIVE_CLAIMED
 *             the timer is running and no session is working, but a session has handed work in
 *             that nobody has reviewed: the work cannot go on until you look at it.
 *   WAITING   in progress, and nothing has been handed in: no work yet, or only work already
 *             accepted on part of the checklist.
 *   CLAIMED   in progress, and a session has handed work in that nobody has reviewed: a claimed
 *             step, or a stepless task whose wrapup claimed it.
 *   ACCEPTED  in progress, and every piece of work has been accepted: the stepless task's review,
 *             or every step on the checklist. It is ready to be moved to Done.
 *   RESTING   not in progress: the mark is the status colour alone.
 *
 * A session at work outranks everything: while one is on a step, Claude has not finished, whatever
 * else is claimed. Then a running timer with a claim pending is LIVE_CLAIMED, a running timer alone
 * LIVE. CLAIMED outranks ACCEPTED: one claimed step on a checklist is still something waiting for
 * you.
 */
export const TASK_MARK_STATES = ['LIVE', 'LIVE_CLAIMED', 'WAITING', 'CLAIMED', 'ACCEPTED', 'RESTING'] as const

export type TaskMarkState = (typeof TASK_MARK_STATES)[number]

export interface TaskMark {
  readonly state: TaskMarkState
  /** Share of the checklist accepted, 0 to 1; always 0 for a task with no checklist. */
  readonly accepted: number
}

export const TASK_MARK_LABEL: Readonly<Record<TaskMarkState, string>> = {
  LIVE: 'A session is running on it',
  LIVE_CLAIMED: 'Timer running, work claimed and awaiting your review',
  WAITING: 'In progress, no work handed in yet',
  CLAIMED: 'Work claimed, awaiting your review',
  ACCEPTED: 'All work accepted, ready to close',
  RESTING: ''
}

/**
 * The mark for one task. `steps` may hold every task's steps: only this task's non-draft ones are
 * read. `timerRunning` is a running time entry, which the task row learns from outside the task.
 */
export function taskMark(task: Task, steps: readonly TaskStep[], timerRunning: boolean): TaskMark {
  const checklist = steps.filter((step) => step.taskId === task.id && step.state !== 'DRAFT')
  const accepted = task.stepCount > 0 ? task.stepsDone / task.stepCount : 0
  const mark = (state: TaskMarkState): TaskMark => ({ state, accepted })

  const stepless = task.reviewActive && task.stepCount === 0
  const sessionRunning = stepless
    ? task.reviewState === 'RUNNING'
    : checklist.some((step) => step.state === 'RUNNING')
  const claimPending = stepless
    ? task.reviewState === 'CLAIMED'
    : checklist.some((step) => step.state === 'CLAIMED')

  if (sessionRunning) return mark('LIVE')
  if (timerRunning) return mark(claimPending ? 'LIVE_CLAIMED' : 'LIVE')

  if (task.status !== 'IN_PROGRESS') return mark('RESTING')

  if (claimPending) return mark('CLAIMED')
  if (stepless) return mark(task.reviewState === 'DONE' ? 'ACCEPTED' : 'WAITING')
  if (task.stepCount > 0 && task.stepsDone === task.stepCount) return mark('ACCEPTED')
  return mark('WAITING')
}
