import { isEpic } from './catalog'
import { oldestFirst } from './task-code'
import type { Task } from './catalog'
import type { TaskId } from '../../../model/branded'

/** How far an epic is, read from the tasks that belong to it. */
export interface EpicProgress {
  readonly total: number
  readonly done: number
  readonly stepCount: number
  readonly stepsDone: number
  /** Tasks with work handed in that nobody has reviewed: a claimed step, or a claimed stepless task. */
  readonly awaitingReview: number
}

/** The tasks that belong to the epic, from the oldest to the newest. */
export function tasksOfEpic(epicId: TaskId, tasks: readonly Task[]): Task[] {
  return tasks.filter((task) => task.epicId === epicId).sort(oldestFirst)
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

/** Every whitespace-separated part of the needle must appear in the epic's title, code or anchor. */
export function matchesEpicQuery(epic: Task, needle: string): boolean {
  const hay = `${epic.title} ${epic.code} ${epic.anchor}`.toLowerCase()
  return needle
    .trim()
    .toLowerCase()
    .split(/\s+/)
    .every((part) => hay.includes(part))
}
