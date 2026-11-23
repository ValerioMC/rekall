import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/decisions/api/decisions.api'
import { useConsoleStore } from '@/stores/console.store'
import { answered, decision, onHold, TASK } from './fixtures'

vi.mock('@/features/decisions/api/decisions.api')

const BLOCK = '<decisions count="2" answered="1" open="1" on-hold="0">\nregion: eu-west-1\n</decisions>'

describe('the decisions slice of the console store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('keeps the decisions of a task in the order the server gave them, with the text a session reads, and shows them for the task in view', async () => {
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [answered('region', 'eu-west-1'), decision('helm')], sessionBlock: BLOCK })
    const store = useConsoleStore()

    await store.loadDecisions(TASK)
    expect(store.selectedTaskDecisions).toEqual([])
    store.selectedTaskId = TASK

    expect(store.selectedTaskDecisions.map((entry) => entry.key)).toEqual(['region', 'helm'])
    expect(store.selectedTaskSessionBlock).toBe(BLOCK)
  })

  it('shows nothing when no task is in view', () => {
    const store = useConsoleStore()

    expect(store.selectedTaskDecisions).toEqual([])
    expect(store.selectedTaskSessionBlock).toBe('')
  })

  it('reads the task again when the server says it changed, replacing what was kept', async () => {
    vi.mocked(api.fetchDecisions).mockResolvedValueOnce({ decisions: [decision('region')], sessionBlock: '' })
    vi.mocked(api.fetchDecisions).mockResolvedValueOnce({ decisions: [onHold('region', 'Asking')], sessionBlock: BLOCK })
    const store = useConsoleStore()
    store.selectedTaskId = TASK

    await store.loadDecisions(TASK)
    await store.loadDecisions(TASK)

    expect(store.selectedTaskDecisions.map((entry) => entry.state)).toEqual(['ON_HOLD'])
  })

  it('adds a decision and reads the task again for the text the server renders', async () => {
    const added = answered('which-namespace', 'billing', { source: 'PERSON' })
    vi.mocked(api.addDecision).mockResolvedValue(added)
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [added], sessionBlock: BLOCK })
    const store = useConsoleStore()
    store.selectedTaskId = TASK

    await store.addDecision(TASK, { topic: null, question: 'Which namespace?', answer: 'billing', stepNumber: null })

    expect(api.addDecision).toHaveBeenCalledWith(TASK, { topic: null, question: 'Which namespace?', answer: 'billing', stepNumber: null })
    expect(store.selectedTaskDecisions).toEqual([added])
    expect(store.selectedTaskSessionBlock).toBe(BLOCK)
  })

  it('swaps an edited decision in place and leaves the others as they were', async () => {
    const before = answered('replicas', '2')
    const other = decision('helm')
    const after = answered('replicas', '3', { previousAnswer: '2', source: 'PERSON' })
    vi.mocked(api.fetchDecisions).mockResolvedValueOnce({ decisions: [before, other], sessionBlock: '' })
    vi.mocked(api.editDecision).mockResolvedValue(after)
    vi.mocked(api.fetchDecisions).mockResolvedValueOnce({ decisions: [after, other], sessionBlock: BLOCK })
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    await store.loadDecisions(TASK)

    await store.editDecision(before, { answer: '3', note: null })

    expect(api.editDecision).toHaveBeenCalledWith(TASK, before.id, { answer: '3', note: null })
    expect(store.selectedTaskDecisions).toEqual([after, other])
  })

  it('takes a removed decision out and reads the task again', async () => {
    const doomed = decision('region')
    vi.mocked(api.fetchDecisions).mockResolvedValueOnce({ decisions: [doomed, decision('helm')], sessionBlock: '' })
    vi.mocked(api.removeDecision).mockResolvedValue(undefined)
    vi.mocked(api.fetchDecisions).mockResolvedValueOnce({ decisions: [decision('helm')], sessionBlock: BLOCK })
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    await store.loadDecisions(TASK)

    await store.removeDecision(doomed)

    expect(api.removeDecision).toHaveBeenCalledWith(doomed.id)
    expect(store.selectedTaskDecisions.map((entry) => entry.key)).toEqual(['helm'])
  })
})
