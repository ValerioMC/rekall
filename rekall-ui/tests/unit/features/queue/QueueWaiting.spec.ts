import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import QueueOrderRow from '@/features/queue/components/QueueOrderRow.vue'
import {
  EMPTY_RUN_QUEUE,
  asRunQueueItemId,
  nextWaiting,
  unqueuedDependencies,
  waitingNote,
  type RunQueueItem,
  type RunQueueWait
} from '@/features/queue/model/runQueue'
import { asTaskId } from '@/model/branded'

function wait(anchor: string, overrides: Partial<RunQueueWait> = {}): RunQueueWait {
  return { anchor, title: anchor, awaitingReview: false, inQueue: true, ...overrides }
}

function item(id: string, waitingFor: RunQueueWait[] = [], state: RunQueueItem['state'] = 'QUEUED'): RunQueueItem {
  return {
    id: asRunQueueItemId(id),
    taskId: asTaskId(`t-${id}`),
    taskTitle: `Task ${id}`,
    taskCode: `t-${id}`,
    projectLabel: 'vega',
    anchor: `project:vega task:t-${id}`,
    position: 0,
    state,
    detail: null,
    startedAt: null,
    finishedAt: null,
    steps: [],
    waitingFor
  }
}

function mountRow(queued: RunQueueItem) {
  return mount(QueueOrderRow, {
    props: { item: queued, position: 1, last: false, live: true, busy: false, canMoveEarlier: false, canMoveLater: false }
  })
}

describe('a queued task that waits for another', () => {
  it('is skipped when the queue looks for the next one to start', () => {
    const queue = { ...EMPTY_RUN_QUEUE, items: [item('a', [wait('project:vega task:b')]), item('b')] }

    expect(nextWaiting(queue)?.id).toBe('b')
  })

  it('says what it waits for, and when that task only awaits your review', () => {
    const waiting = item('a', [wait('project:vega task:b'), wait('project:vega task:c', { awaitingReview: true })])

    expect(waitingNote(waiting)).toBe(
      'Waiting for project:vega task:b, project:vega task:c (claimed, waiting for your review)'
    )
  })

  it('says nothing once it runs, or when nothing holds it back', () => {
    expect(waitingNote(item('a'))).toBeNull()
    expect(waitingNote(item('a', [wait('project:vega task:b')], 'RUNNING'))).toBeNull()
  })

  it('lists the tasks it waits for that are not on the queue', () => {
    const waiting = item('a', [wait('project:vega task:b'), wait('project:vega task:c', { inQueue: false })])

    expect(unqueuedDependencies(waiting).map((missing) => missing.anchor)).toEqual(['project:vega task:c'])
  })

  it('shows the wait on its row, with a warning for a task that is not queued', () => {
    const row = mountRow(item('a', [wait('project:vega task:b'), wait('project:vega task:c', { inQueue: false })]))

    expect(row.get('[data-testid="run-queue-reason"]').text()).toBe('Waiting for project:vega task:b, project:vega task:c')
    expect(row.get('[data-testid="run-queue-unqueued"]').text()).toContain('project:vega task:c is not in the queue')
  })

  it('shows no wait on a task that waits for nothing', () => {
    const row = mountRow(item('a'))

    expect(row.find('[data-testid="run-queue-reason"]').exists()).toBe(false)
    expect(row.find('[data-testid="run-queue-unqueued"]').exists()).toBe(false)
  })
})
