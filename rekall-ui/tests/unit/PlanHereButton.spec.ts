import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import PlanHereButton from '@/components/console/PlanHereButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import type { Terminal } from '@/model/terminal'
import type { TaskId, TaskStepId, TerminalId } from '@/model/branded'

const sendTerminalInput = vi.fn()
const openTerminal = vi.fn()

vi.mock('@/api/terminal.api', () => ({
  fetchTerminals: vi.fn(),
  openTerminal: (...args: unknown[]) => openTerminal(...args),
  closeTerminal: vi.fn(),
  sendTerminalInput: (...args: unknown[]) => sendTerminalInput(...args)
}))

const TASK = 't-1' as TaskId
const ANCHOR = 'project:vega task:report-builder'

function terminal(over: Partial<Terminal> = {}): Terminal {
  return {
    id: 'term-1' as TerminalId,
    taskId: TASK,
    stepId: null as TaskStepId | null,
    anchors: ANCHOR,
    workingDir: '/code/vega',
    projectLabel: 'vega',
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
    skipPermissions: true,
    model: null,
    effort: null,
    live: true,
    startedAt: '2026-09-22T10:00:00Z',
    lastActivityAt: '2026-09-22T10:00:00Z',
    ...over
  }
}

function mountButton(folder: string | null = '/code/vega') {
  return mount(PlanHereButton, { props: { taskId: TASK, anchor: ANCHOR, folder } })
}

describe('the Plan here button', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    sendTerminalInput.mockReset()
    openTerminal.mockReset()
  })

  it('opens a planning session when none is live on the task', async () => {
    openTerminal.mockResolvedValue(terminal({ id: 'term-2' as TerminalId }))
    const terminals = useTerminalStore()
    terminals.terminals = [terminal({ live: false })]
    const openPane = vi.spyOn(useConsoleStore(), 'openTerminal')

    await mountButton().get('[data-testid="plan-here"]').trigger('click')
    await flushPromises()

    expect(openTerminal).toHaveBeenCalledWith(TASK, expect.objectContaining({ mode: 'PLAN' }))
    expect(sendTerminalInput).not.toHaveBeenCalled()
    expect(openPane).toHaveBeenCalled()
    expect(terminals.activeTerminalId).toBe('term-2')
  })

  it('asks for the project folder instead of planning when there is neither a folder nor a session', async () => {
    await mountButton(null).get('[data-testid="plan-here"]').trigger('click')
    await flushPromises()

    expect(openTerminal).not.toHaveBeenCalled()
    expect(sendTerminalInput).not.toHaveBeenCalled()
  })

  it('types the plan command into the live session and brings its terminal forward', async () => {
    sendTerminalInput.mockResolvedValue(undefined)
    const terminals = useTerminalStore()
    terminals.terminals = [terminal()]
    const openPane = vi.spyOn(useConsoleStore(), 'openTerminal')

    await mountButton().get('[data-testid="plan-here"]').trigger('click')
    await flushPromises()

    expect(sendTerminalInput).toHaveBeenCalledWith('term-1', `/rk ${ANCHOR} plan\r`)
    expect(openTerminal).not.toHaveBeenCalled()
    expect(openPane).toHaveBeenCalled()
    expect(terminals.activeTerminalId).toBe('term-1')
  })

  it('leaves the terminal where it was when the command cannot be sent', async () => {
    sendTerminalInput.mockRejectedValue(new Error('gone'))
    const terminals = useTerminalStore()
    terminals.terminals = [terminal()]
    terminals.activeTerminalId = null

    await mountButton().get('[data-testid="plan-here"]').trigger('click')
    await flushPromises()

    expect(terminals.activeTerminalId).toBeNull()
  })

  it('rests at the plain caret while a session is already live, and only swells while it types the plan line', async () => {
    let resolveSend: () => void = () => {}
    sendTerminalInput.mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          resolveSend = resolve
        })
    )
    const terminals = useTerminalStore()
    terminals.terminals = [terminal()]

    const wrapper = mountButton()
    expect(wrapper.get('.session-caret').classes()).not.toContain('session-caret-busy')

    await wrapper.get('[data-testid="plan-here"]').trigger('click')
    expect(wrapper.get('.session-caret').classes()).toContain('session-caret-busy')

    resolveSend()
    await flushPromises()
    expect(wrapper.get('.session-caret').classes()).not.toContain('session-caret-busy')
  })
})
