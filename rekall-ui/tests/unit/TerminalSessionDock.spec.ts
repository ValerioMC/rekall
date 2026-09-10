import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import TerminalSessionDock from '@/components/shell/TerminalSessionDock.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import type { Terminal } from '@/model/terminal'
import type { ProjectId, TaskId, TaskStepId, TerminalId } from '@/model/branded'
import type { Task } from '@/model/catalog'

const api = {
  fetchTerminals: vi.fn(),
  openTerminal: vi.fn(),
  closeTerminal: vi.fn()
}

vi.mock('@/api/terminal.api', () => ({
  fetchTerminals: (...a: unknown[]) => api.fetchTerminals(...a),
  openTerminal: (...a: unknown[]) => api.openTerminal(...a),
  closeTerminal: (...a: unknown[]) => api.closeTerminal(...a)
}))

const routeName = { value: 'projects' }
const push = vi.fn()

vi.mock('vue-router', () => ({
  useRoute: () => ({
    get name() {
      return routeName.value
    }
  }),
  useRouter: () => ({
    currentRoute: {
      get value() {
        return { name: routeName.value }
      }
    },
    push
  })
}))

const TASK = 't-1' as TaskId

const task: Task = {
  id: TASK,
  label: 'report-builder',
  title: 'Report builder',
  status: 'IN_PROGRESS',
  description: null,
  autoWrapup: false,
  wrapupDirective: null,
  projectId: 'p-1' as ProjectId,
  projectLabel: 'vega',
  projectTitle: 'Vega',
  companyName: 'Acme',
  projectRepoFolder: '/code/vega',
  documentCount: 0,
  stepCount: 0,
  stepsDone: 0,
  draftStepCount: 0,
  hasWrapup: false,
  reviewState: 'OPEN',
  reviewActive: true,
  claimedAt: null,
  acceptedAt: null,
  reviewNote: null,
  anchor: 'project:vega task:report-builder',
  updatedAt: '2026-09-10T10:00:00Z'
}

function terminal(over: Partial<Terminal> = {}): Terminal {
  return {
    id: 'term-1' as TerminalId,
    taskId: TASK,
    stepId: null as TaskStepId | null,
    anchors: task.anchor,
    workingDir: '/code/vega',
    projectLabel: 'vega',
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
    skipPermissions: true,
    model: null,
    effort: null,
    live: true,
    startedAt: '2026-09-10T10:00:00Z',
    lastActivityAt: '2026-09-10T10:00:00Z',
    ...over
  }
}

async function mountDock(open: Terminal[]) {
  setActivePinia(createPinia())
  const console = useConsoleStore()
  const terminals = useTerminalStore()
  console.tasks = [task]
  terminals.terminals = open
  const wrapper = mount(TerminalSessionDock)
  await flushPromises()
  return { wrapper, console, terminals }
}

describe('TerminalSessionDock', () => {
  beforeEach(() => {
    Object.values(api).forEach((fn) => fn.mockReset())
    api.closeTerminal.mockResolvedValue(undefined)
    routeName.value = 'projects'
    push.mockReset()
  })

  it('renders nothing when no terminal is live', async () => {
    const { wrapper } = await mountDock([terminal({ live: false })])
    expect(wrapper.find('[data-testid="terminal-dock"]').exists()).toBe(false)
  })

  it('counts the live terminals on the pill', async () => {
    const { wrapper } = await mountDock([
      terminal(),
      terminal({ id: 'term-2' as TerminalId })
    ])
    expect(wrapper.get('[data-testid="terminal-dock-toggle"]').text()).toContain('2 sessions')
  })

  it('reads a lone terminal in the singular', async () => {
    const { wrapper } = await mountDock([terminal()])
    expect(wrapper.get('[data-testid="terminal-dock-toggle"]').text()).toContain('1 session')
  })

  it('jumps to the task, opens the terminal pane and selects the terminal', async () => {
    const { wrapper, console, terminals } = await mountDock([terminal()])

    await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')
    await wrapper.get('[data-testid="terminal-dock-jump"]').trigger('click')
    await flushPromises()

    expect(console.selectedTaskId).toBe(TASK)
    expect(console.paneFocus).toBe('terminal')
    expect(terminals.activeTerminalId).toBe('term-1')
    expect(push).toHaveBeenCalledWith({ name: 'console' })
  })

  it('stops a terminal from its row', async () => {
    const { wrapper } = await mountDock([terminal()])

    await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')
    await wrapper.get('[data-testid="terminal-dock-stop"]').trigger('click')
    await flushPromises()

    expect(api.closeTerminal).toHaveBeenCalledWith('term-1')
  })

  it('stays hidden while the terminal pane is already open', async () => {
    routeName.value = 'console'
    const { wrapper, console } = await mountDock([terminal()])
    console.paneFocus = 'terminal'
    await flushPromises()
    expect(wrapper.find('[data-testid="terminal-dock"]').exists()).toBe(false)
  })
})
