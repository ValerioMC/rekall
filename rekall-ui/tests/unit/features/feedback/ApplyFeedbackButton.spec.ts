import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import ApplyFeedbackButton from '@/features/feedback/components/ApplyFeedbackButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { Feedback } from '@/features/feedback/model/feedback'
import type { ProjectId, TerminalId } from '@/model/branded'
import { entry, revision, TASK } from './fixtures'

const sendTerminalInput = vi.fn()
const openTerminal = vi.fn()

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

function button(mode: 'revise' | 'replan', thread: Feedback[], folder: string | null = '/code/vega') {
  const store = useConsoleStore()
  store.selectedTaskId = TASK
  store.feedbackByTask = { [TASK]: thread }
  return mount(ApplyFeedbackButton, { props: { taskId: TASK, anchor: ANCHOR, folder, mode } })
}

describe('the apply feedback button', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('counts the pending revisions and feedback on the work for Run revision, and only those', () => {
    const wrapper = button('revise', [entry(1), revision(2, 'a', 'b'), entry(3, { kind: 'PLAN' }), entry(4, { state: 'APPLIED' })])

    expect(wrapper.text()).toContain('Run revision')
    expect(wrapper.get('[data-testid="apply-feedback-count"]').text()).toBe('2')
  })

  it('counts the pending feedback on the plan for Replan with feedback', () => {
    const wrapper = button('replan', [entry(1), entry(2, { kind: 'PLAN' }), entry(3, { kind: 'PLAN' })])

    expect(wrapper.text()).toContain('Replan with feedback')
    expect(wrapper.get('[data-testid="apply-feedback-count"]').text()).toBe('2')
  })

  it('is disabled, and says why, with nothing pending', () => {
    const wrapper = button('revise', [entry(1, { state: 'APPLIED' })])

    expect(wrapper.attributes('disabled')).toBeDefined()
    expect(wrapper.attributes('title')).toBe('Nothing is pending')
  })

  it('types the revise line into the live session', async () => {
    sendTerminalInput.mockResolvedValue(undefined)
    useTerminalStore().terminals = [terminal()]
    const wrapper = button('revise', [entry(1)])

    await wrapper.trigger('click')
    await flushPromises()

    expect(sendTerminalInput).toHaveBeenCalledWith('term-1', `/rk ${ANCHOR} revise\r`)
  })

  it('opens a revise terminal when no session is live', async () => {
    openTerminal.mockResolvedValue(terminal())
    const wrapper = button('revise', [entry(1)])

    await wrapper.trigger('click')
    await flushPromises()

    expect(openTerminal).toHaveBeenCalledWith({ taskId: TASK }, expect.objectContaining({ mode: 'REVISE' }))
  })

  it('types the replan line, with no message, into the live session', async () => {
    sendTerminalInput.mockResolvedValue(undefined)
    useTerminalStore().terminals = [terminal()]
    const wrapper = button('replan', [entry(1, { kind: 'PLAN' })])

    await wrapper.trigger('click')
    await flushPromises()

    expect(sendTerminalInput).toHaveBeenCalledWith('term-1', `/rk ${ANCHOR} replan\r`)
  })

  it('does nothing without a folder or a live session, and names what is missing', async () => {
    const wrapper = button('revise', [entry(1)], null)

    await wrapper.trigger('click')
    await flushPromises()

    expect(openTerminal).not.toHaveBeenCalled()
    expect(sendTerminalInput).not.toHaveBeenCalled()
    expect(wrapper.attributes('title')).toContain("Set this project's folder")
  })
})
