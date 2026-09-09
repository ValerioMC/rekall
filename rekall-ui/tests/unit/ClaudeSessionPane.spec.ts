import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ClaudeSessionPane from '@/components/console/ClaudeSessionPane.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { ClaudeMessage, ClaudeSession } from '@/model/claude'
import type { ClaudeMessageId, ClaudeSessionId, ProjectId, TaskId } from '@/model/branded'
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

function message(over: Partial<ClaudeMessage> = {}): ClaudeMessage {
  return {
    id: 'm-1' as ClaudeMessageId,
    sessionId: 's-1' as ClaudeSessionId,
    seq: 0,
    role: 'ASSISTANT',
    content: 'the rows are aggregated in ReportService',
    toolName: null,
    meta: null,
    createdAt: '2026-09-08T10:00:00Z',
    ...over
  }
}

class FakeEventSource {
  readyState = 0
  addEventListener(): void {}
  close(): void {}
}

async function mountPane(withTask = true) {
  setActivePinia(createPinia())
  vi.stubGlobal('EventSource', FakeEventSource)
  const store = useConsoleStore()
  if (withTask) {
    store.tasks = [task]
    store.selectedTaskId = TASK
  }
  const wrapper = mount(ClaudeSessionPane)
  await flushPromises()
  return wrapper
}

describe('ClaudeSessionPane', () => {
  beforeEach(() => {
    Object.values(api).forEach((fn) => fn.mockReset())
    api.fetchClaudeSessions.mockResolvedValue([])
    api.fetchTaskClaudeSessions.mockResolvedValue([])
    api.fetchClaudeTranscript.mockResolvedValue([])
  })

  it('asks for a task when none is selected', async () => {
    const wrapper = await mountPane(false)
    expect(wrapper.text()).toContain('Pick a task to run a session')
  })

  it('shows the start CTA when the task has no sessions', async () => {
    const wrapper = await mountPane()
    expect(wrapper.get('[data-testid="claude-start-first"]').text()).toBe('Start a session')
    expect(wrapper.text()).toContain('Run Claude Code here, not in a terminal')
  })

  it('starts a session from the CTA', async () => {
    api.startClaudeSession.mockResolvedValue(session({ id: 's-new' as ClaudeSessionId }))
    const wrapper = await mountPane()

    await wrapper.get('[data-testid="claude-start-first"]').trigger('click')
    await flushPromises()

    expect(api.startClaudeSession).toHaveBeenCalledWith(TASK, expect.objectContaining({ skipPermissions: expect.any(Boolean) }))
    expect(wrapper.find('[data-testid="claude-transcript"]').exists()).toBe(true)
  })

  it('renders the transcript and an enabled composer for a ready session', async () => {
    api.fetchClaudeSessions.mockResolvedValue([session()])
    api.fetchClaudeTranscript.mockResolvedValue([message()])
    const wrapper = await mountPane()

    expect(wrapper.get('[data-testid="claude-transcript"]').text()).toContain('ReportService')
    expect(wrapper.get('[data-testid="claude-session-status"]').text()).toContain('ready')
    expect((wrapper.get('[data-testid="claude-composer-send"]').element as HTMLButtonElement).disabled).toBe(true)
    await wrapper.get('[data-testid="claude-composer-input"]').setValue('go on')
    expect((wrapper.get('[data-testid="claude-composer-send"]').element as HTMLButtonElement).disabled).toBe(false)
  })

  it('disables the composer and shows why once a session has ended', async () => {
    api.fetchClaudeSessions.mockResolvedValue([
      session({ status: 'EXITED', live: false, detail: 'Stopped from the console.' })
    ])
    const wrapper = await mountPane()

    expect((wrapper.get('[data-testid="claude-composer-input"]').element as HTMLTextAreaElement).disabled).toBe(true)
    expect(wrapper.get('[data-testid="claude-ended-detail"]').text()).toContain('Stopped from the console.')
  })
})
