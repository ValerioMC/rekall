import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import ApplyFeedbackButton from '@/features/feedback/components/ApplyFeedbackButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { Feedback } from '@/features/feedback/model/feedback'
import type { ProjectId, TerminalId } from '@/model/branded'
import { entry, revision, running, TASK } from './fixtures'

const sendTerminalInput = vi.fn()
const openTerminal = vi.fn()
const startPass = vi.fn()

vi.mock('@/features/feedback/api/feedback.api', () => ({
  fetchFeedback: vi.fn(),
  addFeedback: vi.fn(),
  editFeedback: vi.fn(),
  deleteFeedback: vi.fn(),
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

function button(thread: Feedback[], folder: string | null = '/code/vega') {
  const store = useConsoleStore()
  store.selectedTaskId = TASK
  store.feedbackByTask = { [TASK]: thread }
  return mount(ApplyFeedbackButton, { props: { taskId: TASK, anchor: ANCHOR, folder } })
}

describe('the apply feedback button', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    startPass.mockResolvedValue([])
  })

  it('counts every waiting entry whatever its kind, and only those', () => {
    const wrapper = button([entry(1), revision(2, 'a', 'b'), entry(3, { kind: 'PLAN' }), entry(4, { state: 'APPLIED' }), running(5, 1)])

    expect(wrapper.text()).toContain('Run revision')
    expect(wrapper.get('[data-testid="apply-feedback-count"]').text()).toBe('3')
  })

  it('is disabled, and says why, with nothing waiting', () => {
    const wrapper = button([entry(1, { state: 'APPLIED' })])

    expect(wrapper.attributes('disabled')).toBeDefined()
    expect(wrapper.attributes('title')).toBe('Nothing is waiting for a run')
  })

  it('is disabled while a pass runs, and names the pass', () => {
    const wrapper = button([running(1, 3), entry(2)])

    expect(wrapper.attributes('disabled')).toBeDefined()
    expect(wrapper.attributes('title')).toContain('Pass 3 is running')
  })

  it('types the revise line into the live session, then starts the pass', async () => {
    sendTerminalInput.mockResolvedValue(undefined)
    useTerminalStore().terminals = [terminal()]
    const wrapper = button([entry(1), entry(2, { kind: 'PLAN' })])

    await wrapper.trigger('click')
    await flushPromises()

    expect(sendTerminalInput).toHaveBeenCalledWith('term-1', `/rk ${ANCHOR} revise\r`)
    expect(startPass).toHaveBeenCalledWith(TASK)
  })

  it('opens a revise terminal when no session is live, then starts the pass', async () => {
    openTerminal.mockResolvedValue(terminal())
    const wrapper = button([entry(1)])

    await wrapper.trigger('click')
    await flushPromises()

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'REVISE' }))
    expect(startPass).toHaveBeenCalledWith(TASK)
  })

  it('stays on the description: the terminal pane is not opened', async () => {
    openTerminal.mockResolvedValue(terminal())
    const openPane = vi.spyOn(useConsoleStore(), 'openTerminal')
    const wrapper = button([entry(1)])

    await wrapper.trigger('click')
    await flushPromises()

    expect(openPane).not.toHaveBeenCalled()
  })

  it('starts no pass when the session cannot start, so the entries keep waiting', async () => {
    openTerminal.mockRejectedValue(new Error('claude is not installed'))
    const wrapper = button([entry(1)])

    await wrapper.trigger('click')
    await flushPromises()

    expect(startPass).not.toHaveBeenCalled()
  })

  it('does nothing without a folder or a live session, and names what is missing', async () => {
    const wrapper = button([entry(1)], null)

    await wrapper.trigger('click')
    await flushPromises()

    expect(openTerminal).not.toHaveBeenCalled()
    expect(sendTerminalInput).not.toHaveBeenCalled()
    expect(startPass).not.toHaveBeenCalled()
    expect(wrapper.attributes('title')).toContain("Set this project's folder")
  })
})
