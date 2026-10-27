import { TASK_STATUS_ORDER, isEpic } from './catalog'
import type { Task } from './catalog'
import type { TaskId } from '../../../model/branded'

export const EPIC_ACTIVITY_KINDS = [
  'STEP_STARTED',
  'STEP_CLAIMED',
  'STEP_ACCEPTED',
  'TASK_CLAIMED',
  'TASK_ACCEPTED',
  'WRAPUP_WRITTEN',
  'COMMIT_LOGGED'
] as const

export type EpicActivityKind = (typeof EPIC_ACTIVITY_KINDS)[number]

/** One recorded moment on a task of an epic; `subject` is the step's title or the commit's subject. */
export interface EpicActivity {
  readonly taskId: TaskId
  readonly kind: EpicActivityKind
  readonly subject: string | null
  readonly at: string
}

export const EPIC_ACTIVITY_LABEL: Readonly<Record<EpicActivityKind, string>> = {
  STEP_STARTED: 'Step started',
  STEP_CLAIMED: 'Step claimed',
  STEP_ACCEPTED: 'Step accepted',
  TASK_CLAIMED: 'Work claimed',
  TASK_ACCEPTED: 'Work accepted',
  WRAPUP_WRITTEN: 'Wrapup written',
  COMMIT_LOGGED: 'Commit logged'
}

/** How far an epic is, read from the tasks that belong to it. */
export interface EpicProgress {
  readonly total: number
  readonly done: number
  readonly stepCount: number
  readonly stepsDone: number
  /** Tasks with work handed in that nobody has reviewed: a claimed step, or a claimed stepless task. */
  readonly awaitingReview: number
}

/** The tasks that belong to the epic, live work first in the navigator's status order, then by title. */
export function tasksOfEpic(epicId: TaskId, tasks: readonly Task[]): Task[] {
  return tasks
    .filter((task) => task.epicId === epicId)
    .sort(
      (left, right) =>
        TASK_STATUS_ORDER.indexOf(left.status) - TASK_STATUS_ORDER.indexOf(right.status) ||
        left.title.localeCompare(right.title)
    )
}

export function epicProgress(children: readonly Task[], claimedStepTaskIds: ReadonlySet<TaskId>): EpicProgress {
  return {
    total: children.length,
    done: children.filter((task) => task.status === 'DONE').length,
    stepCount: children.reduce((sum, task) => sum + task.stepCount, 0),
    stepsDone: children.reduce((sum, task) => sum + task.stepsDone, 0),
    awaitingReview: children.filter((task) => awaitsReview(task, claimedStepTaskIds)).length
  }
}

function awaitsReview(task: Task, claimedStepTaskIds: ReadonlySet<TaskId>): boolean {
  if (claimedStepTaskIds.has(task.id)) return true
  return task.reviewActive && task.stepCount === 0 && task.reviewState === 'CLAIMED'
}

/** The epics a task of this project can belong to: every epic of the project but the task itself. */
export function epicChoices(projectId: string, tasks: readonly Task[], except: TaskId | null): Task[] {
  return tasks
    .filter((task) => isEpic(task) && task.projectId === projectId && task.id !== except)
    .sort((left, right) => left.title.localeCompare(right.title))
}
