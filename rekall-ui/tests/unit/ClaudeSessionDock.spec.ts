import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ClaudeSessionDock from '@/components/shell/ClaudeSessionDock.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useClaudeStore } from '@/stores/claude.store'
import type { ClaudeSession } from '@/model/claude'
import type { ClaudeSessionId, ProjectId, TaskId } from '@/model/branded'
import type { Task } from '@/model/catalog'

const api = {
  fetchClaudeSessions: vi.fn(),
  fetchTaskClaudeSessions: vi.fn(),
  startClaudeSession: vi.fn(),
  fetchClaudeTranscript: vi.fn(),
  sendClaudePrompt: vi.fn(),
  stopClaudeSession: vi.fn(),
  deleteClaudeSession: vi.fn()
}

vi.mock('@/api/claude.api', () => ({
  fetchClaudeSessions: (...a: unknown[]) => api.fetchClaudeSessions(...a),
  fetchTaskClaudeSessions: (...a: unknown[]) => api.fetchTaskClaudeSessions(...a),
  startClaudeSession: (...a: unknown[]) => api.startClaudeSession(...a),
  fetchClaudeTranscript: (...a: unknown[]) => api.fetchClaudeTranscript(...a),
  sendClaudePrompt: (...a: unknown[]) => api.sendClaudePrompt(...a),
  stopClaudeSession: (...a: unknown[]) => api.stopClaudeSession(...a),
  deleteClaudeSession: (...a: unknown[]) => api.deleteClaudeSession(...a)
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
  stepsDone: 0, draftStepCount: 0,
  hasWrapup: false,
  reviewState: 'OPEN',
  reviewActive: true,
  claimedAt: null,
  acceptedAt: null,
  reviewNote: null,
  anchor: 'project:vega task:report-builder',
  updatedAt: '2026-09-08T10:00:00Z'
}

function session(over: Partial<ClaudeSession> = {}): ClaudeSession {
  return {
    id: 's-1' as ClaudeSessionId,
    taskId: TASK,
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
    projectLabel: 'vega',
    anchor: task.anchor,
    stepId: null,
    anchors: task.anchor,
    workingDir: '/code/vega',
    cliSessionId: null,
    model: null,
    effort: null,
    status: 'READY',
    live: true,
    skipPermissions: true,
    detail: null,
    exitCode: null,
    lastActivityAt: '2026-09-08T10:00:00Z',
    startedAt: '2026-09-08T10:00:00Z',
    endedAt: null,
    updatedAt: '2026-09-08T10:00:00Z',
    ...over
  }
}

async function mountDock(sessions: ClaudeSession[]) {
  setActivePinia(createPinia())
  const console = useConsoleStore()
  const claude = useClaudeStore()
  console.tasks = [task]
  claude.sessions = sessions
  const wrapper = mount(ClaudeSessionDock)
  await flushPromises()
  return { wrapper, console, claude }
}

describe('ClaudeSessionDock', () => {
  beforeEach(() => {
    Object.values(api).forEach((fn) => fn.mockReset())
    api.fetchClaudeTranscript.mockResolvedValue([])
    api.stopClaudeSession.mockResolvedValue(session({ status: 'EXITED', live: false }))
    routeName.value = 'projects'
    push.mockReset()
  })

  it('renders nothing when no session is live', async () => {
    const { wrapper } = await mountDock([session({ status: 'EXITED', live: false })])
    expect(wrapper.find('[data-testid="claude-dock"]').exists()).toBe(false)
  })

  it('counts the live sessions on the pill', async () => {
    const { wrapper } = await mountDock([
      session(),
      session({ id: 's-2' as ClaudeSessionId, status: 'WORKING' })
    ])
    expect(wrapper.get('[data-testid="claude-dock-toggle"]').text()).toContain('2 sessions')
    expect(wrapper.get('[data-testid="claude-dock-toggle"]').text()).toContain('1 working')
  })

  it('reads a lone session in the singular', async () => {
    const { wrapper } = await mountDock([session()])
    expect(wrapper.get('[data-testid="claude-dock-toggle"]').text()).toContain('1 session')
    expect(wrapper.get('[data-testid="claude-dock-toggle"]').text()).not.toContain('working')
  })

  it('jumps to the task, opens the session pane and selects the session', async () => {
    const { wrapper, console, claude } = await mountDock([session()])

    await wrapper.get('[data-testid="claude-dock-toggle"]').trigger('click')
    await wrapper.get('[data-testid="claude-dock-jump"]').trigger('click')
    await flushPromises()

    expect(console.selectedTaskId).toBe(TASK)
    expect(console.paneFocus).toBe('claude')
    expect(claude.activeSessionId).toBe('s-1')
    expect(push).toHaveBeenCalledWith({ name: 'console' })
  })

  it('stops a session from its row', async () => {
    const { wrapper } = await mountDock([session()])

    await wrapper.get('[data-testid="claude-dock-toggle"]').trigger('click')
    await wrapper.get('[data-testid="claude-dock-stop"]').trigger('click')
    await flushPromises()

    expect(api.stopClaudeSession).toHaveBeenCalledWith('s-1')
  })

  it('stays hidden while the session pane is already open', async () => {
    routeName.value = 'console'
    const { wrapper, console } = await mountDock([session()])
    console.paneFocus = 'claude'
    await flushPromises()
    expect(wrapper.find('[data-testid="claude-dock"]').exists()).toBe(false)
  })
})
