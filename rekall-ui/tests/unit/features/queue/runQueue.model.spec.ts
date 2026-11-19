import { describe, expect, it } from 'vitest'
import {
  EMPTY_RUN_QUEUE,
  asRunQueueItemId,
  clockLabel,
  fromLocalInputValue,
  isQueued,
  nextWaiting,
  hasStepsAwaitingReview,
  runProgress,
  stepProgress,
  suggestedStart,
  toLocalInputValue,
  type RunQueue,
  type RunQueueItem,
  type RunQueueItemState,
  type RunQueueStep,
  type RunQueueStepState
} from '@/features/queue/model/runQueue'
import { RunQueueSchema } from '@/features/queue/api/schemas/runQueue.schema'
import { asTaskId } from '@/model/branded'

function item(id: string, state: RunQueueItemState): RunQueueItem {
  return {
    id: asRunQueueItemId(id),
    taskId: asTaskId(`task-${id}`),
    taskTitle: `Task ${id}`,
    taskCode: `task-${id}`,
    projectLabel: 'vega',
    anchor: `project:vega task:task-${id}`,
    position: 0,
    state,
    detail: null,
    startedAt: null,
    finishedAt: null,
    steps: [],
    waitingFor: []
  }
}

function step(title: string, state: RunQueueStepState, current = false): RunQueueStep {
  return { title, state, current }
}

function withSteps(state: RunQueueItemState, ...steps: RunQueueStep[]): RunQueueItem {
  return { ...item('t', state), steps }
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

describe('the step progress of a queued task', () => {
  it('is empty for a task with no checklist', () => {
    expect(stepProgress(item('a', 'RUNNING'))).toEqual({
      total: 0,
      settled: 0,
      currentIndex: null,
      currentTitle: null
    })
  })

  it('counts a claim as settled and numbers the current step among the steps that count', () => {
    const running = withSteps(
      'RUNNING',
      step('One', 'CLAIMED'),
      step('Two', 'DONE'),
      step('Three', 'RUNNING', true),
      step('Four', 'OPEN')
    )

    expect(stepProgress(running)).toEqual({ total: 4, settled: 2, currentIndex: 3, currentTitle: 'Three' })
  })

  it('has no current step once every step is claimed', () => {
    const complete = withSteps('FINISHED', step('One', 'CLAIMED'), step('Two', 'CLAIMED'))

    expect(stepProgress(complete)).toEqual({ total: 2, settled: 2, currentIndex: null, currentTitle: null })
  })

  it('leaves the not-needed steps of a flow that ended early out of the total', () => {
    const ended = withSteps(
      'FINISHED',
      step('One', 'CLAIMED'),
      step('Skipped', 'NOT_NEEDED'),
      step('Two', 'CLAIMED'),
      step('Also skipped', 'NOT_NEEDED')
    )

    expect(stepProgress(ended).total).toBe(2)
    expect(stepProgress(ended).settled).toBe(2)
  })

  it('reads a flow step opened again as not settled and current', () => {
    const looped = withSteps('RUNNING', step('Build', 'CLAIMED'), step('Fix', 'RUNNING', true), step('Test', 'OPEN'))

    expect(stepProgress(looped)).toMatchObject({ total: 3, settled: 1, currentIndex: 2 })
  })

  it('tells a claimed step waiting for review from an accepted one', () => {
    expect(hasStepsAwaitingReview(withSteps('FINISHED', step('One', 'CLAIMED')))).toBe(true)
    expect(hasStepsAwaitingReview(withSteps('FINISHED', step('One', 'DONE')))).toBe(false)
    expect(hasStepsAwaitingReview(item('a', 'FINISHED'))).toBe(false)
  })

  it('rides the run progress of the running task only', () => {
    const queue = queueOf(
      { ...withSteps('FINISHED', step('Old', 'CLAIMED')), id: asRunQueueItemId('a') },
      { ...withSteps('RUNNING', step('A', 'CLAIMED'), step('B', 'RUNNING', true)), id: asRunQueueItemId('b') }
    )

    expect(runProgress(queue).steps).toMatchObject({ total: 2, settled: 1, currentIndex: 2 })
    expect(runProgress(queueOf(item('c', 'QUEUED'))).steps).toBeNull()
  })

  it('parses a frame that carries no steps, and one that does', () => {
    const frame = {
      id: '7a0e5a52-6d52-4a0c-9d8e-0a3a6a1d0001',
      taskId: '7a0e5a52-6d52-4a0c-9d8e-0a3a6a1d0002',
      taskTitle: 'T',
      taskCode: 't',
      projectLabel: 'p',
      anchor: 'project:p task:t',
      position: 0,
      state: 'RUNNING',
      detail: null,
      startedAt: null,
      finishedAt: null
    }
    const queue = {
      ...EMPTY_RUN_QUEUE,
      items: [frame, { ...frame, steps: [{ title: 'A', state: 'NOT_NEEDED', current: false }] }]
    }

    const parsed = RunQueueSchema.parse(queue)

    expect(parsed.items[0]!.steps).toEqual([])
    expect(parsed.items[1]!.steps).toEqual([{ title: 'A', state: 'NOT_NEEDED', current: false }])
  })
})
