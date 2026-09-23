import { describe, expect, it } from 'vitest'
import {
  EMPTY_RUN_QUEUE,
  asRunQueueItemId,
  clockLabel,
  fromLocalInputValue,
  isQueued,
  nextWaiting,
  runProgress,
  suggestedStart,
  toLocalInputValue,
  type RunQueue,
  type RunQueueItem,
  type RunQueueItemState
} from '@/model/runQueue'
import { asTaskId } from '@/model/branded'

function item(id: string, state: RunQueueItemState): RunQueueItem {
  return {
    id: asRunQueueItemId(id),
    taskId: asTaskId(`task-${id}`),
    taskTitle: `Task ${id}`,
    taskLabel: `task-${id}`,
    projectLabel: 'vega',
    anchor: `project:vega task:task-${id}`,
    position: 0,
    state,
    detail: null,
    startedAt: null,
    finishedAt: null
  }
}

function queueOf(...items: RunQueueItem[]): RunQueue {
  return { ...EMPTY_RUN_QUEUE, items }
}

describe('the run queue model', () => {
  it('counts a run: what has had its turn, what is on it, what still waits', () => {
    const queue = queueOf(item('a', 'FINISHED'), item('b', 'SKIPPED'), item('c', 'RUNNING'), item('d', 'QUEUED'))

    const progress = runProgress(queue)

    expect(progress.settled).toBe(2)
    expect(progress.running?.id).toBe('c')
    expect(progress.waiting).toBe(1)
    expect(progress.total).toBe(4)
  })

  it('names the first waiting item as the next one, whatever runs before it', () => {
    expect(nextWaiting(queueOf(item('a', 'FAILED'), item('b', 'QUEUED'), item('c', 'QUEUED')))?.id).toBe('b')
    expect(nextWaiting(queueOf(item('a', 'FINISHED')))).toBeNull()
  })

  it('treats a task as queued while it waits or runs, and free again once it has settled', () => {
    const queue = queueOf(item('a', 'QUEUED'), item('b', 'RUNNING'), item('c', 'FINISHED'))

    expect(isQueued(queue, asTaskId('task-a'))).toBe(true)
    expect(isQueued(queue, asTaskId('task-b'))).toBe(true)
    expect(isQueued(queue, asTaskId('task-c'))).toBe(false)
  })

  it('writes a start time as the clock reads it, with the day only when it is not today', () => {
    const now = new Date(2026, 8, 23, 10, 0).getTime()

    expect(clockLabel(new Date(2026, 8, 23, 14, 5).toISOString(), now)).toMatch(/14[:.]05/)
    expect(clockLabel(new Date(2026, 8, 24, 2, 0).toISOString(), now)).toMatch(/\S+ 02[:.]00/)
    expect(clockLabel(null, now)).toBe('')
  })

  it('round-trips a start time through a datetime-local field, to the minute', () => {
    const iso = new Date(2026, 8, 24, 2, 30).toISOString()

    expect(toLocalInputValue(iso)).toBe('2026-09-24T02:30')
    expect(fromLocalInputValue('2026-09-24T02:30')).toBe(iso)
    expect(fromLocalInputValue('')).toBeNull()
  })

  it('offers a first start time on the hour and at least half an hour out', () => {
    const now = new Date(2026, 8, 23, 10, 40).getTime()

    const offered = new Date(suggestedStart(now))

    expect(offered.getMinutes()).toBe(0)
    expect(offered.getTime() - now).toBeGreaterThanOrEqual(30 * 60_000)
    expect(offered.getHours()).toBe(12)
  })
})
