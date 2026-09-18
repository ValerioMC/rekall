import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ShellDock from '@/components/shell/ShellDock.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useToastStore } from '@/stores/toast.store'
import type { Terminal } from '@/model/terminal'
import type { ProjectId, TaskId, TaskStepId, TerminalId, TimeEntryId } from '@/model/branded'
import type { Task, TimeEntry } from '@/model/catalog'

const terminalApi = {
  fetchTerminals: vi.fn(),
  openTerminal: vi.fn(),
  closeTerminal: vi.fn()
}

vi.mock('@/api/terminal.api', () => ({
  fetchTerminals: (...a: unknown[]) => terminalApi.fetchTerminals(...a),
  openTerminal: (...a: unknown[]) => terminalApi.openTerminal(...a),
  closeTerminal: (...a: unknown[]) => terminalApi.closeTerminal(...a)
}))

const stopTimeEntry = vi.fn()

vi.mock('@/api/time-entries.api', () => ({
  fetchTimeEntries: vi.fn(),
  startTimeEntry: vi.fn(),
  stopTimeEntry: (...a: unknown[]) => stopTimeEntry(...a),
  editTimeEntry: vi.fn(),
  deleteTimeEntry: vi.fn()
}))

const recordLatestCommit = vi.fn()

vi.mock('@/api/commitReference.api', () => ({
  recordLatestCommit: (...a: unknown[]) => recordLatestCommit(...a)
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

/** Reports once on `observe`, so the lane is published without a layout engine. */
class ImmediateResizeObserver implements ResizeObserver {
  constructor(private readonly callback: ResizeObserverCallback) {}

  observe(): void {
    this.callback([], this)
  }

  unobserve(): void {}

  disconnect(): void {}
}

const TASK = 't-1' as TaskId

const task: Task = {
  id: TASK,
  label: 'report-builder',
  title: 'Report builder',
  status: 'IN_PROGRESS',
  description: null,
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
  tagId: null,
  tagName: null,
  tagIcon: null,
  tagColor: null,
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

function running(over: Partial<TimeEntry> = {}): TimeEntry {
  return {
    id: 'time-1' as TimeEntryId,
    taskId: TASK,
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
    projectLabel: 'vega',
    anchor: task.anchor,
    startedAt: '2026-09-10T10:00:00Z',
    stoppedAt: null,
    createdAt: '2026-09-10T10:00:00Z',
    updatedAt: '2026-09-10T10:00:00Z',
    ...over
  }
}

async function mountDock(open: Terminal[], entries: TimeEntry[] = []) {
  setActivePinia(createPinia())
  const console = useConsoleStore()
  const terminals = useTerminalStore()
  console.tasks = [task]
  console.timeEntries = entries
  terminals.terminals = open
  const wrapper = mount(ShellDock, { attachTo: document.body })
  await flushPromises()
  return { wrapper, console, terminals }
}

function lane(): { width: string; height: string } {
  const style = document.documentElement.style
  return {
    width: style.getPropertyValue('--dock-lane-width'),
    height: style.getPropertyValue('--dock-lane-height')
  }
}

describe('ShellDock', () => {
  beforeEach(() => {
    vi.stubGlobal('ResizeObserver', ImmediateResizeObserver)
    vi.spyOn(Element.prototype, 'getBoundingClientRect').mockReturnValue({
      width: 240,
      height: 40
    } as DOMRect)
    Object.values(terminalApi).forEach((fn) => fn.mockReset())
    terminalApi.closeTerminal.mockResolvedValue(undefined)
    stopTimeEntry.mockReset()
    recordLatestCommit.mockReset()
    routeName.value = 'projects'
    push.mockReset()
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    vi.restoreAllMocks()
    document.documentElement.removeAttribute('style')
    document.body.innerHTML = ''
  })

  describe('the bar', () => {
    it('renders nothing when nothing runs and no session is live', async () => {
      const { wrapper } = await mountDock([terminal({ live: false })])
      expect(wrapper.find('[data-testid="shell-dock"]').exists()).toBe(false)
      expect(lane().width).toBe('')
    })

    it('shows only the sessions segment when sessions alone are live', async () => {
      const { wrapper } = await mountDock([terminal()])
      expect(wrapper.find('[data-testid="terminal-dock-toggle"]').exists()).toBe(true)
      expect(wrapper.find('[data-testid="running-dock-toggle"]').exists()).toBe(false)
    })

    it('shows only the running segment when timers alone are live', async () => {
      const { wrapper } = await mountDock([], [running()])
      expect(wrapper.find('[data-testid="running-dock-toggle"]').exists()).toBe(true)
      expect(wrapper.find('[data-testid="terminal-dock-toggle"]').exists()).toBe(false)
    })

    it('publishes the lane for the whole bar whichever segments are present', async () => {
      await mountDock([terminal()])
      expect(lane()).toEqual({ width: '268px', height: '62px' })
    })

    it('counts the live terminals on the sessions segment', async () => {
      const { wrapper } = await mountDock([terminal(), terminal({ id: 'term-2' as TerminalId })])
      expect(wrapper.get('[data-testid="terminal-dock-toggle"]').text()).toContain('2 sessions')
    })

    it('reads a lone terminal in the singular', async () => {
      const { wrapper } = await mountDock([terminal()])
      expect(wrapper.get('[data-testid="terminal-dock-toggle"]').text()).toContain('1 session')
    })

    it('counts the running timers on the running segment', async () => {
      const { wrapper } = await mountDock([], [running(), running({ id: 'time-2' as TimeEntryId })])
      expect(wrapper.get('[data-testid="running-dock-toggle"]').text()).toContain('2 running')
    })

    it('hides the sessions segment while the terminal pane is already open', async () => {
      routeName.value = 'console'
      const { wrapper, console } = await mountDock([terminal()], [running()])
      console.paneFocus = 'terminal'
      await flushPromises()
      expect(wrapper.find('[data-testid="terminal-dock-toggle"]').exists()).toBe(false)
      expect(wrapper.find('[data-testid="running-dock-toggle"]').exists()).toBe(true)
    })
  })

  describe('one panel at a time', () => {
    it('opens the panel of the segment pressed and closes it on a second press', async () => {
      const { wrapper } = await mountDock([terminal()])
      const toggle = wrapper.get('[data-testid="terminal-dock-toggle"]')

      await toggle.trigger('click')
      expect(wrapper.findAll('[data-testid="dock-panel"]')).toHaveLength(1)
      expect(toggle.attributes('aria-expanded')).toBe('true')

      await toggle.trigger('click')
      expect(wrapper.find('[data-testid="dock-panel"]').exists()).toBe(false)
      expect(toggle.attributes('aria-expanded')).toBe('false')
    })

    it('switches to the other panel rather than stacking a second one', async () => {
      const { wrapper } = await mountDock([terminal()], [running()])

      await wrapper.get('[data-testid="running-dock-toggle"]').trigger('click')
      expect(wrapper.find('#dock-panel-running').exists()).toBe(true)

      await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')
      expect(wrapper.findAll('[data-testid="dock-panel"]')).toHaveLength(1)
      expect(wrapper.find('#dock-panel-running').exists()).toBe(false)
      expect(wrapper.find('#dock-panel-sessions').exists()).toBe(true)
      expect(wrapper.get('[data-testid="running-dock-toggle"]').attributes('aria-expanded')).toBe('false')
    })

    it('closes the panel from its own close button', async () => {
      const { wrapper } = await mountDock([terminal()])
      await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')
      await wrapper.get('[data-testid="dock-panel-close"]').trigger('click')
      expect(wrapper.find('[data-testid="dock-panel"]').exists()).toBe(false)
    })

    it('closes the panel on Escape', async () => {
      const { wrapper } = await mountDock([terminal()])
      await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')
      document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
      await flushPromises()
      expect(wrapper.find('[data-testid="dock-panel"]').exists()).toBe(false)
    })

    it('closes the panel on a press anywhere outside the dock', async () => {
      const { wrapper } = await mountDock([terminal()])
      await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')
      document.body.dispatchEvent(new Event('pointerdown', { bubbles: true }))
      await flushPromises()
      expect(wrapper.find('[data-testid="dock-panel"]').exists()).toBe(false)
    })

    it('keeps the panel open on a press inside it', async () => {
      const { wrapper } = await mountDock([terminal()])
      await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')
      wrapper
        .get('[data-testid="dock-panel"]')
        .element.dispatchEvent(new Event('pointerdown', { bubbles: true }))
      await flushPromises()
      expect(wrapper.find('[data-testid="dock-panel"]').exists()).toBe(true)
    })

    it('takes the sessions panel down when its last session ends', async () => {
      const { wrapper, terminals } = await mountDock([terminal()], [running()])
      await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')

      terminals.terminals = [terminal({ live: false })]
      await flushPromises()

      expect(wrapper.find('[data-testid="dock-panel"]').exists()).toBe(false)
      expect(wrapper.find('[data-testid="running-dock-toggle"]').exists()).toBe(true)
    })
  })

  describe('the sessions panel', () => {
    it('jumps to the task, opens the terminal pane, selects the terminal and closes', async () => {
      const { wrapper, console, terminals } = await mountDock([terminal()])

      await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')
      await wrapper.get('[data-testid="terminal-dock-jump"]').trigger('click')
      await flushPromises()

      expect(console.selectedTaskId).toBe(TASK)
      expect(console.paneFocus).toBe('terminal')
      expect(terminals.activeTerminalId).toBe('term-1')
      expect(push).toHaveBeenCalledWith({ name: 'console' })
      expect(wrapper.find('[data-testid="dock-panel"]').exists()).toBe(false)
    })

    it('stops a terminal from its row', async () => {
      const { wrapper } = await mountDock([terminal()])

      await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')
      await wrapper.get('[data-testid="terminal-dock-stop"]').trigger('click')
      await flushPromises()

      expect(terminalApi.closeTerminal).toHaveBeenCalledWith('term-1')
    })

    it('logs the latest commit for a session with the task and step it was opened on', async () => {
      recordLatestCommit.mockResolvedValue({ comment: 'Wire up the button' })
      const { wrapper } = await mountDock([terminal({ stepId: 'step-1' as TaskStepId })])

      await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')
      await wrapper.get('[data-testid="terminal-dock-log-commit"]').trigger('click')
      await flushPromises()

      expect(recordLatestCommit).toHaveBeenCalledWith(TASK, 'step-1')
      expect(useToastStore().toasts[0]?.message).toBe('Logged “Wire up the button”.')
    })

    it('says why nothing was logged when there is no commit to read', async () => {
      recordLatestCommit.mockRejectedValue(new Error('there is no commit yet'))
      const { wrapper } = await mountDock([terminal()])

      await wrapper.get('[data-testid="terminal-dock-toggle"]').trigger('click')
      await wrapper.get('[data-testid="terminal-dock-log-commit"]').trigger('click')
      await flushPromises()

      const toast = useToastStore().toasts[0]
      expect(toast?.kind).toBe('error')
      expect(toast?.message).toBe('there is no commit yet')
    })
  })

  describe('the running panel', () => {
    it('jumps to the task and closes', async () => {
      const { wrapper, console } = await mountDock([], [running()])

      await wrapper.get('[data-testid="running-dock-toggle"]').trigger('click')
      await wrapper.get('[data-testid="running-dock-jump"]').trigger('click')
      await flushPromises()

      expect(console.selectedTaskId).toBe(TASK)
      expect(push).toHaveBeenCalledWith({ name: 'console' })
      expect(wrapper.find('[data-testid="dock-panel"]').exists()).toBe(false)
    })

    it('stops a timer from its row', async () => {
      stopTimeEntry.mockResolvedValue(running({ stoppedAt: '2026-09-10T11:00:00Z' }))
      const { wrapper } = await mountDock([], [running()])

      await wrapper.get('[data-testid="running-dock-toggle"]').trigger('click')
      await wrapper.get('[data-testid="running-dock-stop"]').trigger('click')
      await flushPromises()

      expect(stopTimeEntry).toHaveBeenCalledWith(TASK)
      expect(wrapper.find('[data-testid="shell-dock"]').exists()).toBe(false)
    })
  })
})
