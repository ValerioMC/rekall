import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useDiagramStore } from '@/features/diagram/store/diagram.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { REPORT, VEGA, summary } from './diagram.fixtures'
import type { Terminal } from '@/features/claude/model/terminal'
import type { DiagramId, TerminalId } from '@/model/branded'

const api = { fetchDiagrams: vi.fn(), fetchDiagram: vi.fn(), createDiagram: vi.fn(), deleteDiagram: vi.fn() }

vi.mock('@/features/diagram/api/diagrams.api', () => ({
  fetchDiagrams: (...a: unknown[]) => api.fetchDiagrams(...a),
  fetchDiagram: (...a: unknown[]) => api.fetchDiagram(...a),
  createDiagram: (...a: unknown[]) => api.createDiagram(...a),
  deleteDiagram: (...a: unknown[]) => api.deleteDiagram(...a)
}))

/** Generations start before the fixtures' diagrams are written, unless a test says otherwise. */
const BEFORE_FIXTURES = Date.parse('2026-10-01T09:00:00Z')
const AFTER_FIXTURES = Date.parse('2026-10-01T11:00:00Z')

function opened(id: string): Terminal {
  return { id: id as TerminalId, projectId: VEGA, taskId: null, stepId: null, anchors: 'project:vega', workingDir: '/code/vega', projectLabel: 'vega', taskLabel: null, taskTitle: null, skipPermissions: false, model: null, effort: null, live: true, startedAt: '', lastActivityAt: '' }
}

describe('the diagram store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    sessionStorage.clear()
    Object.values(api).forEach((mock) => mock.mockReset())
    vi.useFakeTimers({ toFake: ['Date'] })
    vi.setSystemTime(BEFORE_FIXTURES)
  })

  afterEach(() => vi.useRealTimers())

  it('loads the gallery without opening a diagram', async () => {
    api.fetchDiagrams.mockResolvedValue([summary('d1', VEGA)])
    const store = useDiagramStore()
    await store.load()
    expect(store.summaries).toHaveLength(1)
    expect(store.selectedId).toBeNull()
    expect(api.fetchDiagram).not.toHaveBeenCalled()
  })

  it('generates on the project alone when no task is named, and on the task when one is', async () => {
    const terminals = useTerminalStore()
    terminals.openForProject = vi.fn().mockResolvedValue(opened('term-p'))
    terminals.openForTask = vi.fn().mockResolvedValue(opened('term-t'))
    const store = useDiagramStore()

    await store.generate({ projectId: VEGA, taskId: null, request: 'checkout', kinds: ['sequence'] })
    expect(terminals.openForProject).toHaveBeenCalledWith(VEGA, expect.objectContaining({ mode: 'GENERATE', request: 'checkout', kinds: ['sequence'] }))

    await store.generate({ projectId: VEGA, taskId: REPORT, request: 'report', kinds: ['state'] })
    expect(terminals.openForTask).toHaveBeenCalledWith(REPORT, expect.objectContaining({ kinds: ['state'] }))
    expect(store.pending.map((generation) => generation.terminalId)).toEqual(['term-t', 'term-p'])
  })

  it('a diagram arriving for a pending generation answers it and is marked as arrived', async () => {
    const terminals = useTerminalStore()
    terminals.openForProject = vi.fn().mockResolvedValue(opened('term-p'))
    const store = useDiagramStore()
    await store.generate({ projectId: VEGA, taskId: null, request: 'checkout', kinds: ['sequence'] })

    await store.applyEvent({ diagramId: 'd9' as DiagramId, diagram: summary('d9', VEGA), deleted: false })

    expect(store.pending).toEqual([])
    expect(store.arrivedId).toBe('d9')
    expect(store.summaries[0]?.id).toBe('d9')
  })

  it('a diagram that arrived while no stream was listening answers its generation on the next load', async () => {
    const terminals = useTerminalStore()
    terminals.openForTask = vi.fn().mockResolvedValue(opened('term-t'))
    const store = useDiagramStore()
    await store.generate({ projectId: VEGA, taskId: REPORT, request: 'report', kinds: ['state'] })
    api.fetchDiagrams.mockResolvedValue([summary('d9', VEGA, REPORT)])

    await store.load()

    expect(store.pending).toEqual([])
    expect(store.arrivedId).toBe('d9')
  })

  it('a diagram written before the generation started does not answer it', async () => {
    vi.setSystemTime(AFTER_FIXTURES)
    const terminals = useTerminalStore()
    terminals.openForProject = vi.fn().mockResolvedValue(opened('term-p'))
    const store = useDiagramStore()
    await store.generate({ projectId: VEGA, taskId: null, request: 'checkout', kinds: ['sequence'] })
    api.fetchDiagrams.mockResolvedValue([summary('d1', VEGA)])

    await store.load()

    expect(store.pending.map((generation) => generation.terminalId)).toEqual(['term-p'])
    expect(store.arrivedId).toBeNull()
  })

  it('a diagram on another task of the project does not answer the generation', async () => {
    const terminals = useTerminalStore()
    terminals.openForProject = vi.fn().mockResolvedValue(opened('term-p'))
    const store = useDiagramStore()
    await store.generate({ projectId: VEGA, taskId: null, request: 'checkout', kinds: ['sequence'] })

    await store.applyEvent({ diagramId: 'd9' as DiagramId, diagram: summary('d9', VEGA, REPORT), deleted: false })

    expect(store.pending).toHaveLength(1)
  })

  it('a deleted diagram leaves the gallery and closes when it was open', async () => {
    api.fetchDiagram.mockResolvedValue({ ...summary('d1', VEGA), sheets: [] })
    const store = useDiagramStore()
    await store.applyEvent({ diagramId: 'd1' as DiagramId, diagram: summary('d1', VEGA), deleted: false })
    await store.open('d1' as DiagramId)

    await store.applyEvent({ diagramId: 'd1' as DiagramId, diagram: null, deleted: true })

    expect(store.summaries).toEqual([])
    expect(store.selectedId).toBeNull()
  })
})
