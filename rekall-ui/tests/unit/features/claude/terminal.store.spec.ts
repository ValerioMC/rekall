import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises } from '@vue/test-utils'
import { ApiError } from '@/api/client'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { ProjectId, TaskId, TaskStepId, TerminalId } from '@/model/branded'

const api = {
  fetchTerminals: vi.fn(),
  openTerminal: vi.fn(),
  closeTerminal: vi.fn(),
  sendTerminalInput: vi.fn()
}

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: (...a: unknown[]) => api.fetchTerminals(...a),
  openTerminal: (...a: unknown[]) => api.openTerminal(...a),
  closeTerminal: (...a: unknown[]) => api.closeTerminal(...a),
  sendTerminalInput: (...a: unknown[]) => api.sendTerminalInput(...a)
}))

const TASK = 't-1' as TaskId

function terminal(over: Partial<Terminal> = {}): Terminal {
  return {
    id: 'term-1' as TerminalId,
    projectId: 'p-1' as ProjectId,
    taskId: TASK,
    stepId: null as TaskStepId | null,
    anchors: 'project:vega task:report-builder',
    workingDir: '/code/vega',
    projectLabel: 'vega',
    taskCode: 'report-builder',
    taskTitle: 'Report builder',
    mode: 'WORK',
    provider: 'CLAUDE',
    skipPermissions: true,
    model: null,
    effort: null,
    kind: 'TERMINAL',
    live: true,
    awaitingPermission: false,
    startedAt: '2026-09-10T10:00:00Z',
    lastActivityAt: '2026-09-10T10:00:00Z',
    connections: [],
    ...over
  }
}

describe('the terminal store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    api.fetchTerminals.mockReset()
    api.openTerminal.mockReset()
    api.closeTerminal.mockReset()
    api.sendTerminalInput.mockReset()
  })

  it('opens one terminal for a task and makes it active', async () => {
    api.openTerminal.mockResolvedValue(terminal())
    const store = useTerminalStore()

    const opened = await store.openForTask(TASK, { skipPermissions: true })

    expect(opened.id).toBe('term-1')
    expect(store.activeTerminalId).toBe('term-1')
    expect(store.terminalForTask(TASK)?.id).toBe('term-1')
  })

  it('lists a chat session it opens without making it the terminal pane\'s active session', async () => {
    api.openTerminal.mockResolvedValue(terminal({ id: 'chat-1' as TerminalId, kind: 'CHAT' }))
    const store = useTerminalStore()

    await store.openForTask(TASK, { skipPermissions: false, kind: 'CHAT' })

    expect(store.terminals.map((session) => session.id)).toEqual(['chat-1'])
    expect(store.activeTerminalId).toBeNull()
  })

  it('sends a run on a task with a live session to the server, which reuses it', async () => {
    api.openTerminal.mockResolvedValue(terminal())
    const store = useTerminalStore()
    store.terminals = [terminal()]
    store.activeTerminalId = null

    const again = await store.openForTask(TASK, { skipPermissions: true })

    expect(again.id).toBe('term-1')
    expect(api.openTerminal).toHaveBeenCalledWith({ taskId: TASK }, { skipPermissions: true })
    expect(store.terminals).toHaveLength(1)
    expect(store.activeTerminalId).toBe('term-1')
  })

  it.each(['PLAN', 'PLAN_FLOW', 'REPLAN', 'REVISE', 'REFINE', 'DISCUSS', 'REWRITE_RESTRUCTURE', 'REWRITE_FIX'] as const)(
    'routes %s through the open endpoint even with a matching live session',
    async (mode) => {
      const store = useTerminalStore()
      const live = terminal({ provider: 'CODEX' })
      const input = { provider: 'CODEX', skipPermissions: false, model: 'gpt-test', effort: 'high' } as const
      store.upsert(live)
      api.openTerminal.mockResolvedValue(live)

      const opened = await store.planForTask(TASK, input, mode, mode === 'REPLAN' ? ' cover the export ' : '')

      expect(opened.id).toBe(live.id)
      expect(api.openTerminal).toHaveBeenCalledWith(
        { taskId: TASK },
        { ...input, mode, request: mode === 'REPLAN' ? 'cover the export' : undefined }
      )
      expect(api.openTerminal).toHaveBeenCalledTimes(1)
      expect(api.sendTerminalInput).not.toHaveBeenCalled()
      expect(store.lastOpened).toEqual(live)
      expect(store.activeTerminalId).toBe(live.id)
    }
  )

  it.each(['CLAUDE', 'CODEX'] as const)('opens %s for refinement without a live session', async (provider) => {
    const store = useTerminalStore()
    api.openTerminal.mockResolvedValue(terminal({ provider, mode: 'REFINE' }))

    const opened = await store.planForTask(TASK, { skipPermissions: false, provider }, 'REFINE')

    expect(opened.provider).toBe(provider)
    expect(api.openTerminal).toHaveBeenCalledTimes(1)
    expect(api.openTerminal).toHaveBeenCalledWith(
      { taskId: TASK }, { skipPermissions: false, provider, mode: 'REFINE' }
    )
  })

  it.each([false, true])('lets the user switch providers after a server conflict with cached session %s', async (cached) => {
    const store = useTerminalStore()
    const existing = terminal({ kind: 'CHAT' })
    const replacement = terminal({ id: 'term-2' as TerminalId, provider: 'CODEX' })
    if (cached) store.upsert(existing)
    const conflict = new ApiError(409, 'Claude Code Chat is already open here.', '')
    api.openTerminal.mockRejectedValueOnce(conflict).mockResolvedValueOnce(replacement)
    api.fetchTerminals.mockResolvedValue([existing])

    const refining = store.planForTask(TASK, { provider: 'CODEX', skipPermissions: false }, 'REFINE')
    await flushPromises()

    expect(store.launchConflict?.session).toEqual(existing)
    expect(api.sendTerminalInput).not.toHaveBeenCalled()
    expect(api.closeTerminal).not.toHaveBeenCalled()
    expect(api.openTerminal).toHaveBeenCalledTimes(1)

    store.resolveLaunchConflict(true)

    await expect(refining).resolves.toEqual(replacement)
    expect(api.closeTerminal).toHaveBeenCalledTimes(1)
    expect(api.closeTerminal).toHaveBeenCalledWith(existing.id)
    expect(api.openTerminal.mock.calls[1]).toEqual(api.openTerminal.mock.calls[0])
    expect(store.launchConflict).toBeNull()
  })

  it('keeps the other provider open when the user declines the switch', async () => {
    const store = useTerminalStore()
    const existing = terminal()
    const conflict = new ApiError(409, 'Claude Code Terminal is already open here.', '')
    api.openTerminal.mockRejectedValue(conflict)
    api.fetchTerminals.mockResolvedValue([existing])

    const pending = store.planForTask(TASK, { provider: 'CODEX', skipPermissions: false }, 'DISCUSS')
    const refused = expect(pending).rejects.toBe(conflict)
    await flushPromises()
    store.resolveLaunchConflict(false)

    await refused
    expect(api.closeTerminal).not.toHaveBeenCalled()
    expect(api.sendTerminalInput).not.toHaveBeenCalled()
    expect(api.openTerminal).toHaveBeenCalledTimes(1)
  })

  it('markEnded drops the terminal out of the live set without removing it', () => {
    const store = useTerminalStore()
    store.terminals = [terminal()]

    store.markEnded('term-1' as TerminalId)

    expect(store.terminals).toHaveLength(1)
    expect(store.terminals[0]?.live).toBe(false)
    expect(store.terminalForTask(TASK)).toBeNull()
  })

  it('close clears local state even when the request fails', async () => {
    api.closeTerminal.mockRejectedValue(new Error('gone'))
    const store = useTerminalStore()
    store.terminals = [terminal()]
    store.activeTerminalId = 'term-1' as TerminalId

    await expect(store.close('term-1' as TerminalId)).rejects.toThrow('gone')

    expect(store.terminals).toHaveLength(0)
    expect(store.activeTerminalId).toBeNull()
  })

  it.each([
    ['CLAUDE', '/rk'],
    ['CODEX', 'Use the $rekall skill with this exact argument line:']
  ] as const)('sendCommand submits the %s prompt to the PTY', async (provider, prompt) => {
    api.sendTerminalInput.mockResolvedValue(undefined)
    const store = useTerminalStore()

    store.upsert(terminal({ provider }))
    await store.sendCommand('term-1' as TerminalId, '/rk project:vega task:report-builder wrapup')

    expect(api.sendTerminalInput).toHaveBeenCalledWith(
      'term-1',
      `${prompt} project:vega task:report-builder wrapup\r`
    )
  })
  it('switches to the session the server opened on the task after the active one ended', async () => {
    vi.useFakeTimers()
    api.fetchTerminals.mockResolvedValue([terminal({ id: 'term-2' as TerminalId })])
    const store = useTerminalStore()
    store.terminals = [terminal({ live: false })]
    store.activeTerminalId = 'term-1' as TerminalId

    const following = store.followNextSession(TASK, 'term-1' as TerminalId)
    await vi.runAllTimersAsync()
    await following

    expect(store.activeTerminalId).toBe('term-2')
    expect(store.terminalForTask(TASK)?.id).toBe('term-2')
    vi.useRealTimers()
  })

  it('leaves the ended session in place when no next one opens', async () => {
    vi.useFakeTimers()
    api.fetchTerminals.mockResolvedValue([])
    const store = useTerminalStore()
    store.terminals = [terminal({ live: false })]
    store.activeTerminalId = 'term-1' as TerminalId

    const following = store.followNextSession(TASK, 'term-1' as TerminalId)
    await vi.runAllTimersAsync()
    await following

    expect(store.activeTerminalId).toBe('term-1')
    expect(store.terminals).toHaveLength(1)
    expect(api.fetchTerminals).toHaveBeenCalledTimes(6)
    vi.useRealTimers()
  })

  it('does not take the pane from a session the person moved to meanwhile', async () => {
    vi.useFakeTimers()
    api.fetchTerminals.mockResolvedValue([terminal({ id: 'term-2' as TerminalId })])
    const store = useTerminalStore()
    store.terminals = [terminal({ live: false })]
    store.activeTerminalId = 'other' as TerminalId

    const following = store.followNextSession(TASK, 'term-1' as TerminalId)
    await vi.runAllTimersAsync()
    await following

    expect(store.activeTerminalId).toBe('other')
    vi.useRealTimers()
  })
})
