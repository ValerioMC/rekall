import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useClaudeStore } from '@/stores/claude.store'
import type { ClaudeMessage, ClaudeSession } from '@/model/claude'
import type { ClaudeMessageId, ClaudeSessionId, TaskId } from '@/model/branded'

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

function session(over: Partial<ClaudeSession> = {}): ClaudeSession {
  return {
    id: 's-1' as ClaudeSessionId,
    taskId: TASK,
    taskLabel: 'report-builder',
    taskTitle: 'Report builder',
    projectLabel: 'vega',
    anchor: 'project:vega task:report-builder',
    stepId: null,
    anchors: 'project:vega task:report-builder',
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
    content: 'hello',
    toolName: null,
    meta: null,
    createdAt: '2026-09-08T10:00:00Z',
    ...over
  }
}

describe('claude.store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    Object.values(api).forEach((fn) => fn.mockReset())
  })

  it('starts a session, selects it and seeds an empty transcript', async () => {
    const store = useClaudeStore()
    api.startClaudeSession.mockResolvedValue(session({ id: 's-9' as ClaudeSessionId }))

    const created = await store.startForTask(TASK, { skipPermissions: true })

    expect(created.id).toBe('s-9')
    expect(store.activeSessionId).toBe('s-9')
    expect(store.sessions).toHaveLength(1)
    expect(store.activeMessages).toEqual([])
    expect(api.fetchClaudeTranscript).not.toHaveBeenCalled()
  })

  it('fetches a transcript the first time a session is selected, then caches it', async () => {
    const store = useClaudeStore()
    store.upsertSession(session())
    api.fetchClaudeTranscript.mockResolvedValue([message()])

    await store.selectSession('s-1' as ClaudeSessionId)
    await store.selectSession('s-1' as ClaudeSessionId)

    expect(api.fetchClaudeTranscript).toHaveBeenCalledTimes(1)
    expect(store.activeMessages).toHaveLength(1)
  })

  it('applyMessage appends in seq order and ignores a duplicate id', () => {
    const store = useClaudeStore()
    store.upsertSession(session())
    void store.selectSession('s-1' as ClaudeSessionId)

    store.applyMessage(message({ id: 'm-2' as ClaudeMessageId, seq: 2, content: 'second' }))
    store.applyMessage(message({ id: 'm-1' as ClaudeMessageId, seq: 1, content: 'first' }))
    store.applyMessage(message({ id: 'm-1' as ClaudeMessageId, seq: 1, content: 'dup' }))

    expect(store.activeMessages.map((m) => m.content)).toEqual(['first', 'second'])
  })

  it('applySession replaces a known session in place', () => {
    const store = useClaudeStore()
    store.upsertSession(session({ status: 'WORKING' }))

    store.applySession(session({ status: 'EXITED', live: false, detail: 'Stopped.' }))

    expect(store.sessions).toHaveLength(1)
    expect(store.sessions[0]!.status).toBe('EXITED')
    expect(store.liveSessions).toHaveLength(0)
  })

  it('remove drops the session, its transcript and the active pointer', async () => {
    const store = useClaudeStore()
    store.upsertSession(session())
    await store.selectSession('s-1' as ClaudeSessionId)
    api.fetchClaudeTranscript.mockResolvedValue([])
    api.deleteClaudeSession.mockResolvedValue(undefined)

    await store.remove('s-1' as ClaudeSessionId)

    expect(store.sessions).toHaveLength(0)
    expect(store.activeSessionId).toBeNull()
    expect(store.messages['s-1']).toBeUndefined()
  })

  it('loadTaskSessions replaces this task\'s rows but keeps other tasks', async () => {
    const store = useClaudeStore()
    store.upsertSession(session({ id: 'other' as ClaudeSessionId, taskId: 't-2' as TaskId }))
    api.fetchTaskClaudeSessions.mockResolvedValue([session({ id: 'fresh' as ClaudeSessionId })])

    await store.loadTaskSessions(TASK)

    expect(store.sessions.map((s) => s.id).sort()).toEqual(['fresh', 'other'])
    expect(store.sessionsForTask(TASK).map((s) => s.id)).toEqual(['fresh'])
  })
})
