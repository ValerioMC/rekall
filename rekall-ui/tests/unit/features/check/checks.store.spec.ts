import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/check/api/checks.api'
import { useChecksStore } from '@/features/check/store/checks.store'
import { PROJECT, STEP, TASK, check, result } from './fixtures'
import type { DoneCheckId } from '@/model/branded'

vi.mock('@/features/check/api/checks.api')

describe('the checks store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('keeps the project checks and each step\'s apart, in their order', async () => {
    vi.mocked(api.fetchChecks).mockResolvedValue([check('b', 1), check('a', 0), check('on-step', 0, STEP)])
    const store = useChecksStore()

    await store.load(PROJECT)

    expect(store.projectChecksOf(PROJECT).map((c) => c.id)).toEqual(['a', 'b'])
    expect(store.stepChecksOf(PROJECT, STEP).map((c) => c.id)).toEqual(['on-step'])
  })

  it('adds a created check last and replaces an edited one in place', async () => {
    vi.mocked(api.fetchChecks).mockResolvedValue([check('a', 0)])
    vi.mocked(api.createProjectCheck).mockResolvedValue(check('b', 1))
    vi.mocked(api.updateCheck).mockResolvedValue(check('a', 0, null, 'Renamed'))
    const store = useChecksStore()
    await store.load(PROJECT)

    await store.createForProject(PROJECT, { label: 'b', command: 'true', timeoutSeconds: 60 })
    await store.update('a' as DoneCheckId, { label: 'Renamed' })

    expect(store.projectChecksOf(PROJECT).map((c) => c.label)).toEqual(['Renamed', 'b'])
  })

  it('creates a step check in the step\'s scope', async () => {
    vi.mocked(api.fetchChecks).mockResolvedValue([])
    vi.mocked(api.createStepCheck).mockResolvedValue(check('form', 0, STEP))
    const store = useChecksStore()
    await store.load(PROJECT)

    await store.createForStep(PROJECT, STEP, { label: 'form', command: 'true', timeoutSeconds: 60 })

    expect(store.stepChecksOf(PROJECT, STEP).map((c) => c.id)).toEqual(['form'])
    expect(store.projectChecksOf(PROJECT)).toEqual([])
  })

  it('shows a move at once and keeps the server\'s order', async () => {
    vi.mocked(api.fetchChecks).mockResolvedValue([check('a', 0), check('b', 1)])
    vi.mocked(api.reorderProjectChecks).mockResolvedValue([check('b', 0), check('a', 1)])
    const store = useChecksStore()
    await store.load(PROJECT)

    await store.move(PROJECT, null, 'b' as DoneCheckId, 'up')

    expect(api.reorderProjectChecks).toHaveBeenCalledWith(PROJECT, ['b', 'a'])
    expect(store.projectChecksOf(PROJECT).map((c) => c.id)).toEqual(['b', 'a'])
  })

  it('puts the old order back when the server refuses a move', async () => {
    vi.mocked(api.fetchChecks).mockResolvedValue([check('a', 0), check('b', 1)])
    vi.mocked(api.reorderProjectChecks).mockRejectedValue(new Error('refused'))
    const store = useChecksStore()
    await store.load(PROJECT)

    await expect(store.move(PROJECT, null, 'b' as DoneCheckId, 'up')).rejects.toThrow('refused')

    expect(store.projectChecksOf(PROJECT).map((c) => c.id)).toEqual(['a', 'b'])
  })

  it('drops a deleted check', async () => {
    vi.mocked(api.fetchChecks).mockResolvedValue([check('a', 0), check('b', 1)])
    vi.mocked(api.deleteCheck).mockResolvedValue(undefined)
    const store = useChecksStore()
    await store.load(PROJECT)

    await store.remove(PROJECT, 'a' as DoneCheckId)

    expect(store.projectChecksOf(PROJECT).map((c) => c.id)).toEqual(['b'])
  })

  it('reads again only what is open when the stream names a change', async () => {
    vi.mocked(api.fetchChecks).mockResolvedValue([])
    vi.mocked(api.fetchTaskCheckResults).mockResolvedValue([result('r1', 'a', true, '2026-11-03T10:00:00Z')])
    const store = useChecksStore()

    await store.applyProjectEvent(PROJECT)
    await store.applyResultEvent(TASK)
    expect(api.fetchChecks).not.toHaveBeenCalled()
    expect(api.fetchTaskCheckResults).not.toHaveBeenCalled()

    await store.load(PROJECT)
    await store.loadResults(TASK)
    await store.applyProjectEvent(PROJECT)
    await store.applyResultEvent(TASK)

    expect(api.fetchChecks).toHaveBeenCalledTimes(2)
    expect(api.fetchTaskCheckResults).toHaveBeenCalledTimes(2)
    expect(store.resultsOf(TASK).map((r) => r.id)).toEqual(['r1'])
  })
})
