import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/feedback/api/feedback.api'
import * as catalogApi from '@/features/catalog/api/catalog.api'
import { useConsoleStore } from '@/stores/console.store'
import { applied, entry, revision, running, TASK } from './fixtures'

vi.mock('@/features/feedback/api/feedback.api')
vi.mock('@/features/catalog/api/catalog.api')

describe('the feedback slice of the console store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('keeps the thread of a task in the order the server gave it, and shows it for the task in view', async () => {
    vi.mocked(api.fetchFeedback).mockResolvedValue([entry(1), applied(2, 'Done.')])
    const store = useConsoleStore()

    await store.loadFeedback(TASK)
    expect(store.selectedTaskFeedback).toEqual([])
    store.selectedTaskId = TASK

    expect(store.selectedTaskFeedback.map((feedback) => feedback.seq)).toEqual([1, 2])
  })

  it('shows an added entry at the end at once, without reading the thread again', async () => {
    vi.mocked(api.fetchFeedback).mockResolvedValue([entry(1)])
    vi.mocked(api.addFeedback).mockResolvedValue(entry(2, { kind: 'PLAN' }))
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    await store.loadFeedback(TASK)

    await store.addFeedback(TASK, 'PLAN', 'Split step 2')

    expect(api.addFeedback).toHaveBeenCalledWith(TASK, 'PLAN', 'Split step 2', null)
    expect(store.selectedTaskFeedback.map((feedback) => feedback.seq)).toEqual([1, 2])
    expect(api.fetchFeedback).toHaveBeenCalledTimes(1)
  })

  it('counts the waiting entries of every kind for the task in view, and not the running or applied ones', async () => {
    vi.mocked(api.fetchFeedback).mockResolvedValue([
      entry(1),
      applied(2, 'Done.'),
      entry(3, { kind: 'PLAN' }),
      revision(4, 'before', 'after'),
      running(5, 1)
    ])
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    await store.loadFeedback(TASK)

    expect(store.waitingCount).toBe(3)
  })

  it('knows which pass is running, and none when no entry is', async () => {
    vi.mocked(api.fetchFeedback).mockResolvedValue([running(1, 2), entry(2)])
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    await store.loadFeedback(TASK)
    expect(store.runningPass).toBe(2)

    vi.mocked(api.fetchFeedback).mockResolvedValue([applied(1, 'Done.')])
    await store.loadFeedback(TASK)
    expect(store.runningPass).toBeNull()
  })

  it('swaps the waiting rows for the entries a pass took, and leaves the entries written since', async () => {
    vi.mocked(api.fetchFeedback).mockResolvedValue([entry(1), entry(2, { kind: 'PLAN' }), entry(3)])
    vi.mocked(api.startPass).mockResolvedValue([running(1, 1), running(2, 1, { kind: 'PLAN' })])
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    await store.loadFeedback(TASK)

    await store.startPass(TASK)

    expect(store.selectedTaskFeedback.map((feedback) => [feedback.seq, feedback.state])).toEqual([
      [1, 'RUNNING'],
      [2, 'RUNNING'],
      [3, 'PENDING']
    ])
    expect(store.waitingCount).toBe(1)
  })

  it('sends the entry it follows along with an added one', async () => {
    vi.mocked(api.fetchFeedback).mockResolvedValue([applied(1, 'Done.')])
    vi.mocked(api.addFeedback).mockResolvedValue(entry(2, { followsSeq: 1 }))
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    await store.loadFeedback(TASK)

    await store.addFeedback(TASK, 'IMPLEMENTATION', 'Still wrong for refunds', 1)

    expect(api.addFeedback).toHaveBeenCalledWith(TASK, 'IMPLEMENTATION', 'Still wrong for refunds', 1)
    expect(store.selectedTaskFeedback[1]?.followsSeq).toBe(1)
  })

  it('swaps an edited entry in place and drops a deleted one', async () => {
    vi.mocked(api.fetchFeedback).mockResolvedValue([entry(1), entry(2)])
    vi.mocked(api.editFeedback).mockResolvedValue(entry(1, { bodyMarkdown: 'Better words' }))
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    await store.loadFeedback(TASK)

    await store.editFeedback(entry(1), 'Better words')
    expect(store.selectedTaskFeedback[0]?.bodyMarkdown).toBe('Better words')

    await store.deleteFeedback(entry(2))
    expect(store.selectedTaskFeedback.map((feedback) => feedback.seq)).toEqual([1])
  })

  it('reads the tasks again after a revision is discarded, since the server put the old description back', async () => {
    vi.mocked(api.fetchFeedback).mockResolvedValue([revision(1, 'before', 'after')])
    vi.mocked(catalogApi.fetchTasks).mockResolvedValue([])
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    await store.loadFeedback(TASK)

    await store.discardRevision(revision(1, 'before', 'after'))

    expect(api.discardRevision).toHaveBeenCalled()
    expect(catalogApi.fetchTasks).toHaveBeenCalledTimes(1)
    expect(store.selectedTaskFeedback).toEqual([])
  })

  it('reads the thread again when the stream says it changed', async () => {
    vi.mocked(api.fetchFeedback).mockResolvedValueOnce([entry(1)]).mockResolvedValueOnce([entry(1), applied(2, 'Done.')])
    const store = useConsoleStore()
    store.selectedTaskId = TASK
    await store.loadFeedback(TASK)

    await store.loadFeedback(TASK)

    expect(store.selectedTaskFeedback).toHaveLength(2)
  })
})
