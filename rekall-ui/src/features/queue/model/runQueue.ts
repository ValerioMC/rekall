import { APP_LOCALE } from '@/common/format/app-locale'
import type { SessionKind } from '@/features/claude/model/terminal'
import type { TaskId } from '@/model/branded'

export const RUN_QUEUE_STATES = ['IDLE', 'SCHEDULED', 'RUNNING', 'HOLDING'] as const
export type RunQueueState = (typeof RUN_QUEUE_STATES)[number]

export const RUN_QUEUE_ITEM_STATES = ['QUEUED', 'RUNNING', 'FINISHED', 'SKIPPED', 'FAILED'] as const
export type RunQueueItemState = (typeof RUN_QUEUE_ITEM_STATES)[number]

export const RUN_QUEUE_STEP_STATES = ['OPEN', 'RUNNING', 'CLAIMED', 'DONE', 'NOT_NEEDED'] as const
export type RunQueueStepState = (typeof RUN_QUEUE_STEP_STATES)[number]

export type QueueBeadState = RunQueueItemState | 'NOT_NEEDED'

/** The run dial's five faces; see RunDial.vue for what each looks like. */
export type RunDialFace = 'empty' | 'ready' | 'scheduled' | 'running' | 'holding'

export type RunQueueItemId = string & { readonly __brand: 'RunQueueItemId' }
export const asRunQueueItemId = (value: string): RunQueueItemId => value as RunQueueItemId

export interface RunQueueStep {
  readonly title: string
  readonly state: RunQueueStepState
  readonly current: boolean
}

/** A task a waiting item depends on and that is not accepted yet. */
export interface RunQueueWait {
  readonly anchor: string
  readonly title: string
  /** Claimed work only waits for the review that accepts it. */
  readonly awaitingReview: boolean
  /** False when that task is not on the list, so nothing in the queue will make it ready. */
  readonly inQueue: boolean
}

export interface RunQueueItem {
  readonly id: RunQueueItemId
  readonly taskId: TaskId
  readonly taskTitle: string
  readonly taskLabel: string
  readonly projectLabel: string
  readonly anchor: string
  readonly position: number
  readonly state: RunQueueItemState
  readonly detail: string | null
  readonly startedAt: string | null
  readonly finishedAt: string | null
  readonly steps: readonly RunQueueStep[]
  readonly waitingFor: readonly RunQueueWait[]
}

export interface RunQueue {
  readonly state: RunQueueState
  readonly startAt: string | null
  /** The usage percentage at which no new task or step starts; null is no ceiling. */
  readonly ceilingPercent: number | null
  readonly skipPermissions: boolean
  readonly model: string | null
  readonly effort: string | null
  readonly sessionKind: SessionKind
  /** How many tasks the queue works at once; from two on, each in a git worktree of its own. */
  readonly parallelSessions: number
  readonly holdUntil: string | null
  readonly holdReason: string | null
  readonly items: readonly RunQueueItem[]
  readonly updatedAt: string
}

export interface RunQueueSettings {
  readonly ceilingPercent: number | null
  readonly skipPermissions: boolean
  readonly model: string | null
  readonly effort: string | null
  readonly sessionKind: SessionKind
  readonly parallelSessions: number
}

/** The numbers of tasks the queue can work at once. */
export const PARALLEL_SESSIONS_CHOICES = [1, 2, 3, 4] as const

/** Where a new ceiling starts: high enough to get work done, low enough to leave room to finish a step. */
export const DEFAULT_CEILING_PERCENT = 85

export const EMPTY_RUN_QUEUE: RunQueue = {
  state: 'IDLE',
  startAt: null,
  ceilingPercent: null,
  skipPermissions: false,
  model: null,
  effort: null,
  sessionKind: 'TERMINAL',
  parallelSessions: 1,
  holdUntil: null,
  holdReason: null,
  items: [],
  updatedAt: new Date(0).toISOString()
}

export function isSettled(state: RunQueueItemState): boolean {
  return state === 'FINISHED' || state === 'SKIPPED' || state === 'FAILED'
}

export function isArmed(state: RunQueueState): boolean {
  return state !== 'IDLE'
}

/** A task's checklist as the run reads it. `total` leaves out not-needed steps; `currentIndex` is 1-based among those. */
export interface StepProgress {
  readonly total: number
  readonly settled: number
  readonly currentIndex: number | null
  readonly currentTitle: string | null
}

/** The run the beacon counts: what has had its turn, what is on it, and what is still waiting. */
export interface RunProgress {
  readonly settled: number
  readonly running: RunQueueItem | null
  readonly waiting: number
  readonly total: number
  readonly steps: StepProgress | null
}

export function isStepSettled(step: RunQueueStep): boolean {
  return step.state === 'CLAIMED' || step.state === 'DONE'
}

/** A claim completes a step for the queue; the person's acceptance comes later, so a claimed step still awaits review. */
export function isAwaitingReview(step: RunQueueStep): boolean {
  return step.state === 'CLAIMED'
}

/** The bead that marks a step: the same marks as a task's, plus the dashed ring of a step the flow did not need. */
export function stepBeadState(step: RunQueueStep): QueueBeadState {
  switch (step.state) {
    case 'OPEN':
      return 'QUEUED'
    case 'RUNNING':
      return 'RUNNING'
    case 'NOT_NEEDED':
      return 'NOT_NEEDED'
    default:
      return 'FINISHED'
  }
}

/** Why a finished task's row says it is done: claimed and waiting for review, or a flow that ended with steps unrun. */
export function finishedNote(item: RunQueueItem): string | null {
  if (item.state !== 'FINISHED') return null
  const notNeeded = item.steps.filter((step) => step.state === 'NOT_NEEDED').length
  if (notNeeded > 0) return `Flow finished, ${notNeeded} ${notNeeded === 1 ? 'step' : 'steps'} not needed`
  if (item.steps.length === 0 || hasStepsAwaitingReview(item)) return 'Claimed, waiting for your review'
  return null
}

export function hasStepsAwaitingReview(item: RunQueueItem): boolean {
  return item.steps.some(isAwaitingReview)
}

export function stepProgress(item: RunQueueItem): StepProgress {
  const counted = item.steps.filter((step) => step.state !== 'NOT_NEEDED')
  const currentPosition = counted.findIndex((step) => step.current)
  return {
    total: counted.length,
    settled: counted.filter(isStepSettled).length,
    currentIndex: currentPosition < 0 ? null : currentPosition + 1,
    currentTitle: currentPosition < 0 ? null : counted[currentPosition]!.title
  }
}

export function runProgress(queue: RunQueue): RunProgress {
  const settled = queue.items.filter((item) => isSettled(item.state)).length
  const running = queue.items.find((item) => item.state === 'RUNNING') ?? null
  const waiting = queue.items.filter((item) => item.state === 'QUEUED').length
  return { settled, running, waiting, total: queue.items.length, steps: running ? stepProgress(running) : null }
}

/** The item the queue will start next: the first one still waiting that waits for no unaccepted task. */
export function nextWaiting(queue: RunQueue): RunQueueItem | null {
  return queue.items.find((item) => item.state === 'QUEUED' && item.waitingFor.length === 0) ?? null
}

/** What a waiting item says it waits for, or null when nothing holds it back. */
export function waitingNote(item: RunQueueItem): string | null {
  if (item.state !== 'QUEUED' || item.waitingFor.length === 0) return null
  const names = item.waitingFor.map((wait) => (wait.awaitingReview ? `${wait.anchor} (claimed, waiting for your review)` : wait.anchor))
  return `Waiting for ${names.join(', ')}`
}

/** The tasks a waiting item depends on that are not on the queue, so it would wait for them for ever. */
export function unqueuedDependencies(item: RunQueueItem): readonly RunQueueWait[] {
  return item.state === 'QUEUED' ? item.waitingFor.filter((wait) => !wait.inQueue) : []
}

/** Whether a task can be added: it is not already waiting or running in the queue. */
export function isQueued(queue: RunQueue, taskId: TaskId): boolean {
  return queue.items.some((item) => item.taskId === taskId && !isSettled(item.state))
}

/**
 * A wall-clock time as the person reads it, with the day only when it is not today: "02:00",
 * "Thu 02:00". Locale-formatted, 24-hour, because a start time is set to the minute.
 */
export function clockLabel(iso: string | null, now: number = Date.now()): string {
  if (!iso) return ''
  const at = new Date(iso)
  if (Number.isNaN(at.getTime())) return ''
  const time = at.toLocaleTimeString(APP_LOCALE, { hour: '2-digit', minute: '2-digit', hour12: false })
  const today = new Date(now)
  const sameDay =
    at.getFullYear() === today.getFullYear() &&
    at.getMonth() === today.getMonth() &&
    at.getDate() === today.getDate()
  if (sameDay) return time
  const day = at.toLocaleDateString(APP_LOCALE, { weekday: 'short' })
  return `${day} ${time}`
}

/** An ISO instant as the value a `datetime-local` input takes, in local time, to the minute. */
export function toLocalInputValue(iso: string): string {
  const at = new Date(iso)
  const pad = (value: number): string => String(value).padStart(2, '0')
  return `${at.getFullYear()}-${pad(at.getMonth() + 1)}-${pad(at.getDate())}T${pad(at.getHours())}:${pad(at.getMinutes())}`
}

/** A `datetime-local` value back to an ISO instant; null when it is empty or not a time. */
export function fromLocalInputValue(value: string): string | null {
  if (!value) return null
  const at = new Date(value)
  return Number.isNaN(at.getTime()) ? null : at.toISOString()
}

/**
 * A sensible first start time to offer: the next top of the hour at least thirty minutes away, so
 * the field never opens on a time that is about to pass.
 */
export function suggestedStart(now: number = Date.now()): string {
  const at = new Date(now + 30 * 60_000)
  at.setMinutes(0, 0, 0)
  at.setHours(at.getHours() + 1)
  return at.toISOString()
}

/** What the person is told after queueing `taskId`: running at once, or how many places wait ahead of it. */
export function queuePlacement(queue: RunQueue, taskId: TaskId): string {
  const item = queue.items.find((candidate) => candidate.taskId === taskId)
  if (!item) return 'Queued'
  if (item.state === 'RUNNING') return 'Running now in the queue'
  const ahead = queue.items.filter(
    (candidate) =>
      candidate.position < item.position && (candidate.state === 'QUEUED' || candidate.state === 'RUNNING')
  ).length
  return `Queued, ${ahead} ahead`
}
