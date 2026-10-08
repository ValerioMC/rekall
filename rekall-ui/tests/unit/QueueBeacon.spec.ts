import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import QueueBeacon from '@/components/queue/QueueBeacon.vue'
import RunDial from '@/components/queue/RunDial.vue'
import { useRunQueueStore } from '@/stores/runQueue.store'
import { EMPTY_RUN_QUEUE, asRunQueueItemId, type RunQueueItem, type RunQueueItemState } from '@/model/runQueue'
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

describe('the run queue beacon', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('reads as an empty queue at rest, and counts what waits once something is lined up', async () => {
    const store = useRunQueueStore()
    const wrapper = mount(QueueBeacon)

    expect(wrapper.find('[data-testid="run-dial"]').attributes('data-face')).toBe('empty')
    expect(wrapper.attributes('aria-label')).toBe('Run queue: empty')

    store.apply({ ...EMPTY_RUN_QUEUE, items: [item('a', 'QUEUED'), item('b', 'QUEUED')], updatedAt: '2026-09-23T10:00:00Z' })
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-testid="run-dial"]').attributes('data-face')).toBe('ready')
    expect(wrapper.text()).toContain('2')
    expect(wrapper.find('[data-testid="queue-beacon-strand"]').exists()).toBe(false)
  })

  it('shows how far the run has got and which task is on it while running', () => {
    const store = useRunQueueStore()
    store.apply({
      ...EMPTY_RUN_QUEUE,
      state: 'RUNNING',
      items: [item('a', 'FINISHED'), item('b', 'RUNNING'), item('c', 'QUEUED')],
      updatedAt: '2026-09-23T10:00:00Z'
    })

    const wrapper = mount(QueueBeacon)

    expect(wrapper.find('[data-testid="queue-beacon-line"]').text()).toContain('2/3')
    expect(wrapper.attributes('aria-label')).toContain('running Report b, 2 of 3')
    expect(wrapper.findAll('[data-testid="queue-bead"]').map((bead) => bead.attributes('data-state'))).toEqual([
      'FINISHED',
      'RUNNING',
      'QUEUED'
    ])
  })

  it('says when a hold lifts, in the warning colour', () => {
    const store = useRunQueueStore()
    store.apply({
      ...EMPTY_RUN_QUEUE,
      state: 'HOLDING',
      ceilingPercent: 85,
      holdUntil: new Date(Date.now() + 3_600_000).toISOString(),
      holdReason: 'Session is at 88%, at or over the 85% ceiling.',
      items: [item('a', 'QUEUED')],
      updatedAt: '2026-09-23T10:00:00Z'
    })

    const wrapper = mount(QueueBeacon)

    expect(wrapper.find('[data-testid="run-dial"]').attributes('data-face')).toBe('holding')
    expect(wrapper.classes()).toContain('text-warn')
    expect(wrapper.attributes('aria-label')).toContain('88%')
  })

  it('opens the panel on click', async () => {
    const store = useRunQueueStore()
    const wrapper = mount(QueueBeacon)

    await wrapper.trigger('click')

    expect(store.panelOpen).toBe(true)
  })
})

describe('the run dial', () => {
  it('points its hand at the start time on a twelve-hour face', () => {
    const wrapper = mount(RunDial, {
      props: { face: 'scheduled', startAt: new Date(2026, 8, 24, 14, 30).toISOString() }
    })

    expect(wrapper.find('[data-testid="run-dial-hand"]').attributes('transform')).toBe('rotate(75 11 11)')
  })

  it('moves only while running', () => {
    expect(mount(RunDial, { props: { face: 'running' } }).find('.dial-comet').exists()).toBe(true)
    expect(mount(RunDial, { props: { face: 'holding', ceilingPercent: 80 } }).find('.dial-comet').exists()).toBe(false)
    expect(mount(RunDial, { props: { face: 'scheduled' } }).find('.dial-core').exists()).toBe(false)
  })
})
