import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import QueueOrderRow from '@/features/queue/components/QueueOrderRow.vue'
import QueueOrderSection from '@/features/queue/components/QueueOrderSection.vue'
import {
  EMPTY_RUN_QUEUE,
  asRunQueueItemId,
  runProgress,
  type RunQueue,
  type RunQueueItem,
  type RunQueueItemState
} from '@/features/queue/model/runQueue'
import { asTaskId } from '@/model/branded'

function item(id: string, state: RunQueueItemState): RunQueueItem {
  return {
    id: asRunQueueItemId(id),
    taskId: asTaskId(`t-${id}`),
    taskTitle: `Report ${id}`,
    taskLabel: `t-${id}`,
    projectLabel: 'vega',
    anchor: `project:vega task:t-${id}`,
    position: 0,
    state,
    detail: null,
    startedAt: null,
    finishedAt: null
  }
}

function mountSection(...items: RunQueueItem[]) {
  const queue: RunQueue = { ...EMPTY_RUN_QUEUE, items }
  return mount(QueueOrderSection, {
    props: { queue, progress: runProgress(queue), busy: false },
    global: { stubs: { QueueTaskPicker: true } }
  })
}

describe('the order of the run queue', () => {
  it('says so when nothing is queued', () => {
    const section = mountSection()

    expect(section.find('[data-testid="run-queue-empty"]').exists()).toBe(true)
    expect(section.find('[data-testid="run-queue-clear"]').exists()).toBe(false)
  })

  it('counts moves among the waiting tasks only, so a finished one never shifts the places', async () => {
    const section = mountSection(item('a', 'FINISHED'), item('b', 'QUEUED'), item('c', 'QUEUED'))
    const rows = section.findAllComponents(QueueOrderRow)
    const rowAt = (index: number) => {
      const row = rows[index]
      if (!row) throw new Error(`no row at ${index}`)
      return row
    }

    rowAt(2).vm.$emit('move', -1)
    rowAt(1).vm.$emit('move', -1)
    rowAt(1).vm.$emit('move', 1)
    await section.vm.$nextTick()

    expect(section.emitted('move')).toEqual([[expect.objectContaining({ id: 'c' }), 0], [expect.objectContaining({ id: 'b' }), 1]])
  })

  it('offers to clear the finished ones and asks the panel to do it', async () => {
    const section = mountSection(item('a', 'FINISHED'), item('b', 'QUEUED'))

    await section.get('[data-testid="run-queue-clear"]').trigger('click')

    expect(section.emitted('clear')).toHaveLength(1)
  })
})
