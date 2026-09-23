import type { TaskId } from '@/model/branded'

export const RUN_QUEUE_STATES = ['IDLE', 'SCHEDULED', 'RUNNING', 'HOLDING'] as const
export type RunQueueState = (typeof RUN_QUEUE_STATES)[number]

export const RUN_QUEUE_ITEM_STATES = ['QUEUED', 'RUNNING', 'FINISHED', 'SKIPPED', 'FAILED'] as const
export type RunQueueItemState = (typeof RUN_QUEUE_ITEM_STATES)[number]

/** The run dial's five faces; see RunDial.vue for what each looks like. */
export type RunDialFace = 'empty' | 'ready' | 'scheduled' | 'running' | 'holding'

export type RunQueueItemId = string & { readonly __brand: 'RunQueueItemId' }
export const asRunQueueItemId = (value: string): RunQueueItemId => value as RunQueueItemId

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
}

export interface RunQueue {
  readonly state: RunQueueState
  readonly startAt: string | null
  /** The usage percentage at which no new task or step starts; null is no ceiling. */
  readonly ceilingPercent: number | null
  readonly skipPermissions: boolean
  readonly model: string | null
  readonly effort: string | null
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
}

/** Where a new ceiling starts: high enough to get work done, low enough to leave room to finish a step. */
export const DEFAULT_CEILING_PERCENT = 85

export const EMPTY_RUN_QUEUE: RunQueue = {
  state: 'IDLE',
  startAt: null,
  ceilingPercent: null,
  skipPermissions: false,
  model: null,
  effort: null,
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

/** The run the beacon counts: what has had its turn, what is on it, and what is still waiting. */
export interface RunProgress {
  readonly settled: number
  readonly running: RunQueueItem | null
  readonly waiting: number
  readonly total: number
}

export function runProgress(queue: RunQueue): RunProgress {
  const settled = queue.items.filter((item) => isSettled(item.state)).length
  const running = queue.items.find((item) => item.state === 'RUNNING') ?? null
  const waiting = queue.items.filter((item) => item.state === 'QUEUED').length
  return { settled, running, waiting, total: queue.items.length }
}

/** The item the queue will start next: the first one still waiting. */
export function nextWaiting(queue: RunQueue): RunQueueItem | null {
  return queue.items.find((item) => item.state === 'QUEUED') ?? null
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
  const time = at.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', hour12: false })
  const today = new Date(now)
  const sameDay =
    at.getFullYear() === today.getFullYear() &&
    at.getMonth() === today.getMonth() &&
    at.getDate() === today.getDate()
  if (sameDay) return time
  const day = at.toLocaleDateString(undefined, { weekday: 'short' })
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
