import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import RevisionRunBar from '@/features/feedback/components/RevisionRunBar.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { Feedback } from '@/features/feedback/model/feedback'
import type { ProjectId, TerminalId } from '@/model/branded'
import { applied, entry, running, TASK } from './fixtures'

const sendTerminalInput = vi.fn()
const openTerminal = vi.fn()
const startPass = vi.fn()

vi.mock('@/features/feedback/api/feedback.api', () => ({
  fetchFeedback: vi.fn(),
  retargetFeedback: vi.fn(),
  discardRevision: vi.fn(),
  startPass: (...args: unknown[]) => startPass(...args)
}))

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: vi.fn(),
  openTerminal: (...args: unknown[]) => openTerminal(...args),
  closeTerminal: vi.fn(),
  sendTerminalInput: (...args: unknown[]) => sendTerminalInput(...args)
}))

const ANCHOR = 'project:vega task:report-builder'

function terminal(): Terminal {
  return {
    id: 'term-1' as TerminalId,
    projectId: 'p-1' as ProjectId,
    taskId: TASK,
    stepId: null,
    anchors: ANCHOR,
    workingDir: '/code/vega',
    projectLabel: 'vega',
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
    mode: 'WORK',
    skipPermissions: true,
    model: null,
    effort: null,
    kind: 'TERMINAL',
    live: true,
    awaitingPermission: false,
    startedAt: '2026-11-04T10:00:00Z',
    lastActivityAt: '2026-11-04T10:00:00Z',
    connections: []
  }
}

function bar(thread: Feedback[], revisionsShown = false, folder: string | null = '/code/vega') {
  const store = useConsoleStore()
  store.selectedTaskId = TASK
  store.feedbackByTask = { [TASK]: thread }
  const retarget = vi.spyOn(store, 'retargetFeedback').mockResolvedValue()
  const wrapper = mount(RevisionRunBar, { props: { taskId: TASK, anchor: ANCHOR, folder, revisionsShown } })
  return { wrapper, retarget, store }
}

function runKey(wrapper: ReturnType<typeof bar>['wrapper']) {
  return wrapper.get('[data-testid="apply-feedback"]')
}

async function press(wrapper: ReturnType<typeof bar>['wrapper']): Promise<void> {
  await runKey(wrapper).trigger('click')
  await vi.advanceTimersByTimeAsync(400)
  await flushPromises()
}

describe('the bar that runs a waiting revision', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.useFakeTimers()
    startPass.mockResolvedValue([])
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('counts the revisions waiting and holds the one Run revision key', () => {
    const { wrapper } = bar([entry(1), entry(2)])

    expect(wrapper.get('[data-testid="revision-run-summary"]').text()).toBe('2 revisions waiting')
    expect(wrapper.findAll('[data-testid="apply-feedback"]')).toHaveLength(1)
    expect(wrapper.get('[data-testid="apply-feedback-count"]').text()).toBe('2')
  })

  it('is not there when nothing waits and no pass runs', () => {
    expect(bar([applied(1, 'Done.')]).wrapper.find('[data-testid="revision-run-bar"]').exists()).toBe(false)
  })

  it('shows what the waiting revision is applied to', () => {
    expect(bar([entry(1)]).wrapper.get('[data-testid="revision-target-IMPLEMENTATION"]').attributes('aria-pressed')).toBe('true')
    expect(bar([entry(1, { kind: 'PLAN' })]).wrapper.get('[data-testid="revision-target-PLAN"]').attributes('aria-pressed')).toBe('true')
  })

  it('points the waiting revision at the other target when it is chosen', async () => {
    const { wrapper, retarget } = bar([entry(1), entry(2, { state: 'APPLIED' })])

    await wrapper.get('[data-testid="revision-target-PLAN"]').trigger('click')
    await flushPromises()

    expect(retarget).toHaveBeenCalledTimes(1)
    expect(retarget).toHaveBeenCalledWith(entry(1), 'PLAN')
  })

  it('offers to see the changes unless they are already on screen', async () => {
    const closed = bar([entry(1)]).wrapper
    await closed.get('[data-testid="revision-run-view"]').trigger('click')
    expect(closed.emitted('showRevisions')).toHaveLength(1)

    expect(bar([entry(1)], true).wrapper.find('[data-testid="revision-run-view"]').exists()).toBe(false)
  })

  it('types the revise line into the live session, then starts the pass', async () => {
    sendTerminalInput.mockResolvedValue(undefined)
    useTerminalStore().terminals = [terminal()]
    const { wrapper } = bar([entry(1), entry(2, { kind: 'PLAN' })])

    await press(wrapper)

    expect(sendTerminalInput).toHaveBeenCalledWith('term-1', `/rk ${ANCHOR} revise\r`)
    expect(startPass).toHaveBeenCalledWith(TASK)
  })

  it('opens a revise terminal when no session is live, then starts the pass, staying on the description', async () => {
    openTerminal.mockResolvedValue(terminal())
    const { wrapper, store } = bar([entry(1)])
    const openPane = vi.spyOn(store, 'openTerminal')

    await press(wrapper)

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'REVISE' }))
    expect(startPass).toHaveBeenCalledWith(TASK)
    expect(openPane).not.toHaveBeenCalled()
  })

  it('starts no pass when the session cannot start, so the entries keep waiting and the key returns', async () => {
    openTerminal.mockRejectedValue(new Error('claude is not installed'))
    const { wrapper } = bar([entry(1)])

    await press(wrapper)

    expect(startPass).not.toHaveBeenCalled()
    expect(runKey(wrapper).attributes('data-phase')).toBe('idle')
  })

  it('does nothing without a folder or a live session, and names what is missing', async () => {
    const { wrapper } = bar([entry(1)], false, null)

    await press(wrapper)

    expect(openTerminal).not.toHaveBeenCalled()
    expect(startPass).not.toHaveBeenCalled()
    expect(runKey(wrapper).attributes('title')).toContain("Set this project's folder")
  })

  it('turns the key into Working while a pass runs, and keeps a revision written meanwhile from being run', async () => {
    const { wrapper, store } = bar([running(1, 3), entry(2)])

    expect(runKey(wrapper).attributes('data-phase')).toBe('working')
    expect(wrapper.get('[data-testid="revision-run-summary"]').text()).toBe('Pass 3 is running, 1 revision waiting')
    expect(wrapper.find('[data-testid="revision-target-PLAN"]').exists()).toBe(false)
    await runKey(wrapper).trigger('click')
    await flushPromises()
    expect(openTerminal).not.toHaveBeenCalled()
    expect(store.runningPass).toBe(3)
  })

  it('stamps a check when the pass is answered, then goes away, or offers the run again if a revision waits', async () => {
    const finished = bar([running(1, 1)])
    finished.store.feedbackByTask = { [TASK]: [applied(1, 'Fixed.', { pass: 1 })] }
    await flushPromises()

    expect(runKey(finished.wrapper).attributes('data-phase')).toBe('done')
    expect(finished.wrapper.find('.run-seal').exists()).toBe(true)
    await vi.advanceTimersByTimeAsync(1300)
    expect(finished.wrapper.find('[data-testid="revision-run-bar"]').exists()).toBe(false)

    setActivePinia(createPinia())
    const again = bar([running(1, 1), entry(2)])
    again.store.feedbackByTask = { [TASK]: [applied(1, 'Fixed.', { pass: 1 }), entry(2)] }
    await flushPromises()
    expect(runKey(again.wrapper).attributes('data-phase')).toBe('done')
    await vi.advanceTimersByTimeAsync(1300)
    expect(runKey(again.wrapper).attributes('data-phase')).toBe('idle')
    expect(again.wrapper.get('[data-testid="revision-run-summary"]').text()).toBe('1 revision waiting')
  })
})
