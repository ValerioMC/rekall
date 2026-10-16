import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, nextTick } from 'vue'
import QueueOrderRow from '@/features/queue/components/QueueOrderRow.vue'
import QueueStatusLine from '@/features/queue/components/QueueStatusLine.vue'
import { useQueueSettingsDraft, type QueueSettingsDraft } from '@/features/queue/composables/useQueueSettingsDraft'
import {
  EMPTY_RUN_QUEUE,
  asRunQueueItemId,
  runProgress,
  type RunQueue,
  type RunQueueItem,
  type RunQueueItemState,
  type RunQueueSettings
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

function queue(over: Partial<RunQueue>): RunQueue {
  return { ...EMPTY_RUN_QUEUE, ...over }
}

function mountRow(state: RunQueueItemState, over: Record<string, unknown> = {}) {
  return mount(QueueOrderRow, {
    props: { item: item('a', state), position: 2, last: false, live: false, busy: false, canMoveEarlier: true, canMoveLater: true, ...over }
  })
}

describe('a run queue row', () => {
  it('lets a waiting task move both ways and come off, saying which way it moved', async () => {
    const row = mountRow('QUEUED')

    await row.get('[data-testid="run-queue-up"]').trigger('click')
    await row.get('[data-testid="run-queue-down"]').trigger('click')
    await row.get('[data-testid="run-queue-remove"]').trigger('click')

    expect(row.emitted('move')).toEqual([[-1], [1]])
    expect(row.emitted('remove')).toHaveLength(1)
    expect(row.text()).toContain('Waiting')
  })

  it('will not move the first task earlier or the last one later', () => {
    const row = mountRow('QUEUED', { canMoveEarlier: false, canMoveLater: false })

    expect(row.get('[data-testid="run-queue-up"]').attributes('disabled')).toBeDefined()
    expect(row.get('[data-testid="run-queue-down"]').attributes('disabled')).toBeDefined()
  })

  it('offers the running task to watch instead of moves, and a finished one nothing', async () => {
    const running = mountRow('RUNNING')
    const finished = mountRow('FINISHED')

    await running.get('[data-testid="run-queue-watch"]').trigger('click')

    expect(running.emitted('watch')).toHaveLength(1)
    expect(running.find('[data-testid="run-queue-up"]').exists()).toBe(false)
    expect(finished.find('button').exists()).toBe(false)
  })

  it('draws no rail under the last bead', () => {
    expect(mountRow('QUEUED', { last: true }).find('.w-px').exists()).toBe(false)
    expect(mountRow('QUEUED').find('.w-px').exists()).toBe(true)
  })
})

describe('the run queue status line', () => {
  const now = Date.parse('2026-09-23T10:00:00Z')

  function mountLine(current: RunQueue) {
    return mount(QueueStatusLine, { props: { queue: current, progress: runProgress(current), now } })
  }

  it('says nothing while the queue is idle', () => {
    expect(mountLine(queue({})).find('[data-testid="run-queue-status"]').exists()).toBe(false)
  })

  it('names the running task and counts what is done and waiting', () => {
    const line = mountLine(queue({ state: 'RUNNING', items: [item('a', 'FINISHED'), item('b', 'RUNNING'), item('c', 'QUEUED')] }))

    expect(line.text()).toContain('Running Report b')
    expect(line.text()).toContain('1 of 3 done, 1 waiting.')
  })

  it('says why a holding queue waits and when it resumes', () => {
    const line = mountLine(
      queue({ state: 'HOLDING', holdUntil: '2026-09-23T11:00:00Z', holdReason: 'Session usage is at 90%.', items: [item('a', 'QUEUED')] })
    )

    expect(line.text()).toContain('Holding until')
    expect(line.text()).toContain('Session usage is at 90%.')
    expect(line.get('[data-testid="run-queue-status"]').classes()).toContain('bg-warn-soft')
  })
})

describe('the run queue settings draft', () => {
  let draft: QueueSettingsDraft
  let save: ReturnType<typeof vi.fn<(settings: RunQueueSettings) => Promise<void>>>
  let report: ReturnType<typeof vi.fn<(error: unknown) => void>>

  function mountDraft(current: RunQueue) {
    return mount(
      defineComponent({
        setup() {
          draft = useQueueSettingsDraft(current, save, report)
          return () => null
        }
      })
    )
  }

  beforeEach(() => {
    vi.useFakeTimers()
    save = vi.fn(async () => undefined)
    report = vi.fn()
  })

  afterEach(() => vi.useRealTimers())

  it('starts from the queue, reading no model and no ceiling as the defaults', () => {
    mountDraft(queue({ ceilingPercent: null, model: null, effort: 'high', skipPermissions: true }))

    expect(draft.ceilingOn.value).toBe(false)
    expect(draft.model.value).toBe('default')
    expect(draft.effort.value).toBe('high')
    expect(draft.skipPermissions.value).toBe(true)
  })

  it('saves once after a burst of changes settles, sending defaults back as null', async () => {
    mountDraft(queue({}))

    draft.ceilingOn.value = true
    draft.ceiling.value = 70
    await nextTick()
    vi.advanceTimersByTime(349)
    expect(save).not.toHaveBeenCalled()
    vi.advanceTimersByTime(1)

    expect(save).toHaveBeenCalledTimes(1)
    expect(save).toHaveBeenCalledWith({ ceilingPercent: 70, skipPermissions: false, model: null, effort: null })
  })

  it('flushes a change still waiting when the panel closes', async () => {
    const wrapper = mountDraft(queue({}))

    draft.model.value = 'opus'
    await nextTick()
    wrapper.unmount()

    expect(save).toHaveBeenCalledWith(expect.objectContaining({ model: 'opus' }))
  })

  it('settles by saving now when nothing is on its way, and reports a failed save instead of throwing', async () => {
    save.mockRejectedValueOnce(new Error('offline'))
    mountDraft(queue({}))

    await draft.settle()

    expect(save).toHaveBeenCalledTimes(1)
    expect(report).toHaveBeenCalledWith(new Error('offline'))
  })
})
