import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import PlanHereButton from '@/components/console/PlanHereButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import type { Terminal } from '@/model/terminal'
import type { TaskId, TaskStepId, TerminalId } from '@/model/branded'

const sendTerminalInput = vi.fn()

vi.mock('@/api/terminal.api', () => ({
  fetchTerminals: vi.fn(),
  openTerminal: vi.fn(),
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

function mountButton() {
  return mount(PlanHereButton, { props: { taskId: TASK, anchor: ANCHOR } })
}

describe('the Plan here button', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    sendTerminalInput.mockReset()
  })

  it('is not there without a live session on the task', () => {
    useTerminalStore().terminals = [terminal({ live: false })]

    expect(mountButton().find('[data-testid="plan-here"]').exists()).toBe(false)
  })

  it('types the plan command into the live session and brings its terminal forward', async () => {
    sendTerminalInput.mockResolvedValue(undefined)
    const terminals = useTerminalStore()
    terminals.terminals = [terminal()]
    const console_ = useConsoleStore()
    const openTerminal = vi.spyOn(console_, 'openTerminal')

    await mountButton().get('[data-testid="plan-here"]').trigger('click')
    await flushPromises()

    expect(sendTerminalInput).toHaveBeenCalledWith('term-1', `/rk ${ANCHOR} plan\r`)
    expect(openTerminal).toHaveBeenCalled()
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
})
