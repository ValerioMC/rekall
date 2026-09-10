import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useTerminalStore } from '@/stores/terminal.store'
import type { Terminal } from '@/model/terminal'
import type { TaskId, TaskStepId, TerminalId } from '@/model/branded'

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

const TASK = 't-1' as TaskId

function terminal(over: Partial<Terminal> = {}): Terminal {
  return {
    id: 'term-1' as TerminalId,
    taskId: TASK,
    stepId: null as TaskStepId | null,
    anchors: 'project:vega task:report-builder',
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

describe('the terminal store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    api.fetchTerminals.mockReset()
    api.openTerminal.mockReset()
    api.closeTerminal.mockReset()
  })

  it('opens one terminal for a task and makes it active', async () => {
    api.openTerminal.mockResolvedValue(terminal())
    const store = useTerminalStore()

    const opened = await store.openForTask(TASK, { skipPermissions: true })

    expect(opened.id).toBe('term-1')
    expect(store.activeTerminalId).toBe('term-1')
    expect(store.terminalForTask(TASK)?.id).toBe('term-1')
  })

  it('refocuses the live terminal instead of opening a second one', async () => {
    const store = useTerminalStore()
    store.terminals = [terminal()]
    store.activeTerminalId = null

    const again = await store.openForTask(TASK, { skipPermissions: true })

    expect(again.id).toBe('term-1')
    expect(api.openTerminal).not.toHaveBeenCalled()
    expect(store.activeTerminalId).toBe('term-1')
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
})
