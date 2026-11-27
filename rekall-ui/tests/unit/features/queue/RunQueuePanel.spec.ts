import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import { preferredProvider, setCodexBypass, setPreferredProvider } from '@/common/config/session-launch'
import { useCodexStore } from '@/features/codex/store/codex.store'
import { installStorage } from '../../support/memoryStorage'
import RunQueuePanel from '@/features/queue/components/RunQueuePanel.vue'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Terminal } from '@/features/claude/model/terminal'
import { useConsoleStore } from '@/stores/console.store'
import {
  EMPTY_RUN_QUEUE,
  asRunQueueItemId,
  type RunQueue,
  type RunQueueItem,
  type RunQueueItemState
} from '@/features/queue/model/runQueue'
import { asTaskId } from '@/model/branded'
import type { Task } from '@/features/catalog/model/catalog'

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

vi.mock('@/features/queue/api/runQueue.api', () => api)

vi.mock('@/features/claude/api/claude.api', () => ({
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
    provider: 'CLAUDE',
    claudeOptions: null,
    codexOptions: null,
    taskId: asTaskId(`t-${id}`),
    taskTitle: `Report ${id}`,
    taskCode: `t-${id}`,
    projectLabel: 'vega',
    anchor: `project:vega task:t-${id}`,
    position: 0,
    state,
    detail: null,
    startedAt: null,
    finishedAt: null,
    steps: [],
    waitingFor: []
  }
}

let clock = 0
function queue(over: Partial<RunQueue>): RunQueue {
  clock += 1
  return { ...EMPTY_RUN_QUEUE, updatedAt: new Date(Date.UTC(2026, 8, 23, 10, 0, clock)).toISOString(), ...over }
}

let router: Router

async function mountPanel(current: RunQueue) {
  setActivePinia(createPinia())
  router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { render: () => null } }] })
  api.fetchRunQueue.mockResolvedValue(current)
  useRunQueueStore().apply(current)
  const wrapper = mount(RunQueuePanel, { attachTo: document.body, global: { plugins: [router] } })
  await flushPromises()
  return wrapper
}

function launchDialog(): DOMWrapper<Element> {
  return new DOMWrapper(document.querySelector('[data-testid="queue-launch-dialog"]')!)
}

async function selectTask(wrapper: Awaited<ReturnType<typeof mountPanel>>) {
  useConsoleStore().tasks = [
    { id: asTaskId('t-queue'), title: 'Report', code: 'VG-1', anchor: 'task:VG-1', status: 'TODO', stepCount: 0, draftStepCount: 0, stepsDone: 0, reviewState: 'OPEN' }
  ] as Task[]
  const input = wrapper.get('[data-testid="queue-task-input"]')
  await input.setValue('report')
  await input.trigger('keydown', { key: 'Enter' })
  await flushPromises()
  return input
}

describe('the run queue panel', () => {
  beforeEach(() => {
    Object.values(api).forEach((mock) => mock.mockReset())
    document.body.innerHTML = ''
    installStorage()
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

    expect(document.body.textContent).toContain('Closes the Claude Code session working on Report a')
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
    setPreferredProvider('CODEX')
    const claudeOptions = { model: 'sonnet', effort: 'high' }
    const saved = queue({ items: [...current.items, { ...item('z', 'QUEUED'), claudeOptions }] })
    api.enqueueTask.mockResolvedValue(saved)
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
    expect(api.enqueueTask).not.toHaveBeenCalled()
    const dialog = launchDialog()
    await dialog.get('input[value="CLAUDE"]').setValue(true)
    await dialog.get('[data-testid="queue-launch-model"] select').setValue('sonnet')
    await dialog.get('[data-testid="queue-launch-effort"] select').setValue('high')
    await dialog.get('form').trigger('submit')
    await flushPromises()

    expect(api.enqueueTask).toHaveBeenCalledTimes(1)
    expect(api.enqueueTask).toHaveBeenCalledWith({ taskId: 't-z', provider: 'CLAUDE', claudeOptions })
    expect(api.startRunQueue).not.toHaveBeenCalled()
    expect(preferredProvider()).toBe('CODEX')
    wrapper.unmount()

    const reopened = await mountPanel(saved)
    const row = reopened.findAll('[data-testid="run-queue-item"]')[1]!
    expect(row.get('[data-testid="run-queue-provider"]').text()).toBe('Claude Code')
    expect(row.get('[data-testid="run-queue-model"]').text()).toBe('Sonnet')
    expect(row.get('[data-testid="run-queue-effort"]').text()).toBe('High')
    reopened.unmount()
  })

  it('retains the order and returns focus to the picker on cancel', async () => {
    const wrapper = await mountPanel(queue({ items: [item('a', 'QUEUED')] }))
    const input = await selectTask(wrapper)

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await flushPromises()

    expect(wrapper.emitted('close')).toBeUndefined()
    expect(document.querySelector('[data-testid="queue-launch-dialog"]')).toBeNull()
    expect(document.activeElement).toBe(input.element)
    expect(api.enqueueTask).not.toHaveBeenCalled()
    expect(useRunQueueStore().queue.items.map(({ id }) => id)).toEqual(['a'])
    wrapper.unmount()
  })

  it('retries a failed Codex catalog and limits effort to the chosen model', async () => {
    setPreferredProvider('CODEX')
    setCodexBypass(true)
    const wrapper = await mountPanel(queue({}))
    const codex = useCodexStore()
    const load = vi.spyOn(codex, 'loadModels').mockRejectedValueOnce(new Error('Catalog unavailable'))
      .mockImplementationOnce(async () => {
        codex.models = [
          { model: 'gpt-first', displayName: 'First', supportedReasoningEfforts: [{ reasoningEffort: 'high' }] },
          { model: 'gpt-second', displayName: 'Second', supportedReasoningEfforts: [{ reasoningEffort: 'low' }] }
        ]
      })
    await selectTask(wrapper)
    const dialog = launchDialog()
    const confirm = dialog.get('[data-testid="queue-launch-confirm"]')
    expect(dialog.text()).toContain('Could not load Codex models.')
    expect(confirm.attributes('disabled')).toBeDefined()
    const retry = dialog.findAll('button').find((button) => button.text() === 'Retry')!
    await retry.trigger('click')
    await flushPromises()
    expect(load).toHaveBeenCalledTimes(2)

    const model = dialog.get<HTMLSelectElement>('[data-testid="queue-launch-model"] select')
    const effort = dialog.get<HTMLSelectElement>('[data-testid="queue-launch-effort"] select')
    expect(document.activeElement).toBe(model.element)
    expect(effort.attributes('disabled')).toBeDefined()
    await model.setValue('gpt-first')
    await effort.setValue('high')
    await model.setValue('gpt-second')
    expect([...effort.element.options].map(({ value }) => value)).toEqual(['default', 'low'])
    expect(effort.element.value).toBe('default')
    await effort.setValue('low')

    api.enqueueTask.mockRejectedValueOnce(new Error('Try the queue again.')).mockResolvedValueOnce(queue({}))
    await dialog.get('form').trigger('submit')
    await flushPromises()
    expect(dialog.get('[role="alert"]').text()).toBe('Try the queue again.')
    expect(effort.element.value).toBe('low')
    await dialog.get('form').trigger('submit')
    await flushPromises()
    expect(api.enqueueTask).toHaveBeenLastCalledWith({
      taskId: 't-queue', provider: 'CODEX', codexOptions: { model: 'gpt-second', effort: 'low', skipPermissions: true }
    })
    expect(api.enqueueTask).toHaveBeenCalledTimes(2)
    wrapper.unmount()
  })

  it('identifies the session that blocked a queued task', async () => {
    const blocked = {
      ...item('a', 'FAILED'),
      provider: 'CODEX' as const,
      detail: 'Claude Code Chat is already open on this task. Close it and queue the task again.'
    }
    const wrapper = await mountPanel(queue({ items: [blocked] }))
    useTerminalStore().terminals = [{
      taskId: blocked.taskId, provider: 'CLAUDE', kind: 'CHAT', live: true
    } as Terminal]
    await flushPromises()

    expect(wrapper.get('[data-testid="run-queue-provider"]').text()).toBe('Codex')
    expect(wrapper.get('[data-testid="run-queue-reason"]').text()).toContain('Claude Code Chat')
    expect(wrapper.findAll('button').some((button) => button.text() === 'Open Claude Code Chat')).toBe(true)
    wrapper.unmount()
  })

  it('opens the task of a row on click, whatever its state, and closes itself', async () => {
    const current = queue({ state: 'RUNNING', items: [item('a', 'FINISHED'), item('b', 'QUEUED')] })
    const wrapper = await mountPanel(current)

    await wrapper.findAll('[data-testid="run-queue-open"]')[0]!.trigger('click')

    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/task/t-a')
    expect(wrapper.emitted('close')).toHaveLength(1)
    wrapper.unmount()
  })
})
