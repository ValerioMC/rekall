import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import RunQueuePanel from '@/components/queue/RunQueuePanel.vue'
import { useRunQueueStore } from '@/stores/runQueue.store'
import { useConsoleStore } from '@/stores/console.store'
import {
  EMPTY_RUN_QUEUE,
  asRunQueueItemId,
  type RunQueue,
  type RunQueueItem,
  type RunQueueItemState
} from '@/model/runQueue'
import { asTaskId } from '@/model/branded'
import type { Task } from '@/model/catalog'

const api = vi.hoisted(() => ({
  fetchRunQueue: vi.fn(),
  saveRunQueueSettings: vi.fn(),
  enqueueTask: vi.fn(),
  dequeueItem: vi.fn(),
  moveQueueItem: vi.fn(),
  clearSettledItems: vi.fn(),
  startRunQueue: vi.fn(),
  stopRunQueue: vi.fn()
}))

vi.mock('@/api/runQueue.api', () => api)

vi.mock('@/api/claude.api', () => ({
  fetchClaudeUsage: vi.fn(async () => ({
    status: 'OK',
    limits: [{ key: 'session', label: 'Session', percent: 40, severity: 'NORMAL', resetsAt: null }],
    fetchedAt: '2026-09-23T10:00:00Z',
    retryAt: null
  }))
}))

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

let clock = 0
function queue(over: Partial<RunQueue>): RunQueue {
  clock += 1
  return { ...EMPTY_RUN_QUEUE, updatedAt: new Date(Date.UTC(2026, 8, 23, 10, 0, clock)).toISOString(), ...over }
}

async function mountPanel(current: RunQueue) {
  setActivePinia(createPinia())
  api.fetchRunQueue.mockResolvedValue(current)
  useRunQueueStore().apply(current)
  const wrapper = mount(RunQueuePanel, { attachTo: document.body })
  await flushPromises()
  return wrapper
}

describe('the run queue panel', () => {
  beforeEach(() => {
    Object.values(api).forEach((mock) => mock.mockReset())
    document.body.innerHTML = ''
  })

  it('invites a first task when the queue is empty, and cannot start', async () => {
    const wrapper = await mountPanel(queue({}))

    expect(wrapper.find('[data-testid="run-queue-empty"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="run-queue-start"]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('lists the run in order, lets a waiting task move and come off, and not the running one', async () => {
    const current = queue({ state: 'RUNNING', items: [item('a', 'FINISHED'), item('b', 'RUNNING'), item('c', 'QUEUED'), item('d', 'QUEUED')] })
    api.dequeueItem.mockResolvedValue(queue({ ...current, items: current.items.slice(0, 3) }))
    api.moveQueueItem.mockResolvedValue(queue(current))
    const wrapper = await mountPanel(current)

    const rows = wrapper.findAll('[data-testid="run-queue-item"]')
    expect(rows.map((row) => row.attributes('data-state'))).toEqual(['FINISHED', 'RUNNING', 'QUEUED', 'QUEUED'])
    expect(rows[1]!.find('[data-testid="run-queue-remove"]').exists()).toBe(false)
    expect(rows[1]!.find('[data-testid="run-queue-watch"]').exists()).toBe(true)

    await rows[3]!.find('[data-testid="run-queue-up"]').trigger('click')
    await flushPromises()
    expect(api.moveQueueItem).toHaveBeenCalledWith('d', 0)

    await rows[3]!.find('[data-testid="run-queue-remove"]').trigger('click')
    await flushPromises()
    expect(api.dequeueItem).toHaveBeenCalledWith('d')
    expect(wrapper.findAll('[data-testid="run-queue-item"]')).toHaveLength(3)
    wrapper.unmount()
  })

  it('saves the settings, then starts at once', async () => {
    const current = queue({ items: [item('a', 'QUEUED')] })
    api.saveRunQueueSettings.mockResolvedValue(queue(current))
    api.startRunQueue.mockResolvedValue(queue({ ...current, state: 'RUNNING' }))
    const wrapper = await mountPanel(current)

    const start = wrapper.find('[data-testid="run-queue-start"]')
    expect(start.text()).toBe('Start now')
    await start.trigger('click')
    await flushPromises()

    expect(api.saveRunQueueSettings).toHaveBeenCalledTimes(1)
    expect(api.startRunQueue).toHaveBeenCalledWith(null)
    wrapper.unmount()
  })

  it('schedules for the time picked, and refuses one that has passed', async () => {
    const current = queue({ items: [item('a', 'QUEUED')] })
    api.saveRunQueueSettings.mockResolvedValue(queue(current))
    api.startRunQueue.mockResolvedValue(queue({ ...current, state: 'SCHEDULED' }))
    const wrapper = await mountPanel(current)

    await wrapper.find('[data-testid="run-queue-when-later"]').trigger('click')
    const field = wrapper.find('[data-testid="run-queue-start-at"]')

    await field.setValue('2020-01-01T02:00')
    expect(wrapper.text()).toContain('That time has passed')
    expect(wrapper.find('[data-testid="run-queue-start"]').attributes('disabled')).toBeDefined()

    const later = new Date(Date.now() + 5 * 3_600_000)
    later.setSeconds(0, 0)
    const pad = (value: number): string => String(value).padStart(2, '0')
    await field.setValue(
      `${later.getFullYear()}-${pad(later.getMonth() + 1)}-${pad(later.getDate())}T${pad(later.getHours())}:${pad(later.getMinutes())}`
    )
    const start = wrapper.find('[data-testid="run-queue-start"]')
    expect(start.text()).toMatch(/^Schedule for /)
    await start.trigger('click')
    await flushPromises()

    expect(api.startRunQueue).toHaveBeenCalledWith(later.toISOString())
    wrapper.unmount()
  })

  it('saves a ceiling once it is switched on, at the default line', async () => {
    vi.useFakeTimers()
    const current = queue({ items: [item('a', 'QUEUED')] })
    api.saveRunQueueSettings.mockResolvedValue(queue({ ...current, ceilingPercent: 85 }))
    const wrapper = await mountPanel(current)

    await wrapper.find('[data-testid="run-queue-ceiling-on"] input').setValue(true)
    await vi.advanceTimersByTimeAsync(400)

    expect(api.saveRunQueueSettings).toHaveBeenCalledWith(
      expect.objectContaining({ ceilingPercent: 85, skipPermissions: false })
    )
    vi.useRealTimers()
    wrapper.unmount()
  })

  it('asks before stopping a session mid-run, and names what it closes', async () => {
    const current = queue({ state: 'RUNNING', items: [item('a', 'RUNNING')] })
    api.stopRunQueue.mockResolvedValue(queue({ ...current, state: 'IDLE', items: [item('a', 'QUEUED')] }))
    const wrapper = await mountPanel(current)

    await wrapper.find('[data-testid="run-queue-stop"]').trigger('click')
    await flushPromises()

    expect(document.body.textContent).toContain('Closes the Claude session working on Report a')
    expect(api.stopRunQueue).not.toHaveBeenCalled()

    const confirm = Array.from(document.body.querySelectorAll('button')).find(
      (button) => button.textContent?.trim() === 'Stop the queue' && button.closest('[role="alertdialog"]')
    )
    confirm!.click()
    await flushPromises()

    expect(api.stopRunQueue).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('adds a task picked by typing, and does not offer one already waiting', async () => {
    const current = queue({ items: [item('a', 'QUEUED')] })
    api.enqueueTask.mockResolvedValue(queue(current))
    setActivePinia(createPinia())
    const wrapper = await mountPanel(current)
    useConsoleStore().tasks = [
      { id: asTaskId('t-a'), title: 'Report a', label: 't-a', anchor: 'project:vega task:t-a', status: 'TODO', stepCount: 0, draftStepCount: 0, stepsDone: 0, reviewState: 'OPEN' },
      { id: asTaskId('t-z'), title: 'Report z', label: 't-z', anchor: 'project:vega task:t-z', status: 'TODO', stepCount: 3, draftStepCount: 0, stepsDone: 1, reviewState: 'OPEN' }
    ] as unknown as Task[]

    const input = wrapper.find('[data-testid="queue-task-input"]')
    await input.trigger('focus')
    await input.setValue('report')
    const options = wrapper.findAll('[data-testid="queue-task-option"]')
    expect(options).toHaveLength(1)
    expect(options[0]!.text()).toContain('1 of 3 steps done')

    await input.trigger('keydown', { key: 'Enter' })
    await flushPromises()
    expect(api.enqueueTask).toHaveBeenCalledWith('t-z')
    wrapper.unmount()
  })
})
