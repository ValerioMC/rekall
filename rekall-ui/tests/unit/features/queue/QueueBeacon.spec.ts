import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import QueueBeacon from '@/features/queue/components/QueueBeacon.vue'
import RunDial from '@/features/queue/components/RunDial.vue'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'
import {
  EMPTY_RUN_QUEUE,
  asRunQueueItemId,
  type RunQueueItem,
  type RunQueueItemState,
  type RunQueueStep
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
    finishedAt: null,
    steps: []
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
    expect(wrapper.attributes('aria-label')).toContain('running Report b, task 2 of 3')
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

function stepped(...steps: RunQueueStep[]): RunQueueItem {
  return { ...item('b', 'RUNNING'), steps }
}

function runningWith(running: RunQueueItem): void {
  useRunQueueStore().apply({
    ...EMPTY_RUN_QUEUE,
    state: 'RUNNING',
    items: [item('a', 'FINISHED'), running, item('c', 'QUEUED')],
    updatedAt: '2026-09-23T10:00:00Z'
  })
}

describe('the run queue beacon with steps', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('adds the current step to the figure and to the full sentence', () => {
    runningWith(
      stepped(
        { title: 'One', state: 'CLAIMED', current: false },
        { title: 'Two', state: 'CLAIMED', current: false },
        { title: 'Three', state: 'RUNNING', current: true },
        { title: 'Four', state: 'OPEN', current: false }
      )
    )

    const wrapper = mount(QueueBeacon)

    expect(wrapper.find('[data-testid="queue-beacon-step"]').text()).toBe('step 3/4')
    expect(wrapper.attributes('aria-label')).toBe('Run queue: running Report b, task 2 of 3, step 3 of 4')
    expect(wrapper.attributes('title')).toBe(wrapper.attributes('aria-label'))
    expect(wrapper.find('[data-testid="queue-beacon-step"]').classes()).toContain('hidden')
  })

  it('says nothing of steps for a task with no checklist', () => {
    runningWith(stepped())

    const wrapper = mount(QueueBeacon)

    expect(wrapper.find('[data-testid="queue-beacon-step"]').exists()).toBe(false)
    expect(wrapper.attributes('aria-label')).toBe('Run queue: running Report b, task 2 of 3')
  })

  it('draws the ring from the running task\'s steps', () => {
    runningWith(
      stepped(
        { title: 'One', state: 'CLAIMED', current: false },
        { title: 'Two', state: 'RUNNING', current: true }
      )
    )

    const wrapper = mount(QueueBeacon)

    expect(wrapper.findAll('[data-testid="run-dial-slice"]')).toHaveLength(2)
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

  it('draws one slice per counted step: the settled filled, the current one pulsing, the rest hairline', () => {
    const wrapper = mount(RunDial, { props: { face: 'running', steps: { total: 5, settled: 2 } } })

    const slices = wrapper.findAll('[data-testid="run-dial-slice"]')
    expect(slices.map((slice) => slice.attributes('data-slice'))).toEqual([
      'settled',
      'settled',
      'current',
      'pending',
      'pending'
    ])
    expect(slices.map((slice) => slice.classes('stroke-accent'))).toEqual([true, true, true, false, false])
    expect(slices.map((slice) => slice.classes('stroke-border'))).toEqual([false, false, false, true, true])
    expect(wrapper.find('.dial-comet').exists()).toBe(false)
  })

  it('keeps the comet when the task has no steps', () => {
    for (const steps of [null, { total: 0, settled: 0 }]) {
      const wrapper = mount(RunDial, { props: { face: 'running', steps } })

      expect(wrapper.find('.dial-comet').exists()).toBe(true)
      expect(wrapper.find('.dial-core').exists()).toBe(true)
      expect(wrapper.findAll('[data-testid="run-dial-slice"]')).toHaveLength(0)
    }
  })

  it('draws a continuous arc above twelve steps', () => {
    const wrapper = mount(RunDial, { props: { face: 'running', steps: { total: 20, settled: 5 } } })

    expect(wrapper.findAll('[data-testid="run-dial-slice"]')).toHaveLength(0)
    expect(wrapper.find('[data-testid="run-dial-arc"]').exists()).toBe(true)
    expect(wrapper.find('.dial-comet').exists()).toBe(false)
  })

  it('settles once when a step completes, and not when it first draws', async () => {
    const wrapper = mount(RunDial, { props: { face: 'running', steps: { total: 3, settled: 1 } } })
    expect(wrapper.classes()).not.toContain('settle')

    await wrapper.setProps({ steps: { total: 3, settled: 2 } })
    await new Promise((resolve) => requestAnimationFrame(resolve))

    expect(wrapper.classes()).toContain('settle')
  })
})
