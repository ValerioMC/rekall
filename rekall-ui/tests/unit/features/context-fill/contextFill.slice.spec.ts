import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/context-fill/api/context-fill.api'
import { useConsoleStore } from '@/stores/console.store'
import { fill, STEP, TASK } from './fixtures'

vi.mock('@/features/context-fill/api/context-fill.api')

describe('the context fill slice of the console store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('loads the fills of a task and answers for the task and for a step', async () => {
    vi.mocked(api.fetchContextFills).mockResolvedValue([fill(null, 40), fill(STEP, 70)])
    const store = useConsoleStore()

    await store.loadContextFills(TASK)

    expect(api.fetchContextFills).toHaveBeenCalledWith(TASK)
    expect(store.contextFillOf({ taskId: TASK, stepId: null })?.percent).toBe(40)
    expect(store.contextFillOf({ taskId: TASK, stepId: STEP })?.percent).toBe(70)
  })

  it('has no fill for a task nothing was loaded for', () => {
    const store = useConsoleStore()

    expect(store.contextFillOf({ taskId: TASK, stepId: null })).toBeNull()
    expect(store.selectedTaskContextFills).toEqual([])
  })

  it('replaces the whole list of a task when a frame arrives, and leaves other tasks alone', async () => {
    vi.mocked(api.fetchContextFills).mockResolvedValue([fill(null, 40), fill(STEP, 70)])
    const store = useConsoleStore()
    await store.loadContextFills(TASK)
    const otherTask = 'aaaaaaaa-0000-4000-8000-000000000002' as typeof TASK
    store.applyContextFillEvent({ taskId: otherTask, fills: [fill(null, 10)] })

    store.applyContextFillEvent({ taskId: TASK, fills: [fill(null, 55)] })

    expect(store.contextFillOf({ taskId: TASK, stepId: null })?.percent).toBe(55)
    expect(store.contextFillOf({ taskId: TASK, stepId: STEP })).toBeNull()
    expect(store.contextFillOf({ taskId: otherTask, stepId: null })?.percent).toBe(10)
  })

  it('shows the fills of the task in view', async () => {
    vi.mocked(api.fetchContextFills).mockResolvedValue([fill(null, 40)])
    const store = useConsoleStore()
    await store.loadContextFills(TASK)
    store.selectedTaskId = TASK

    expect(store.selectedTaskContextFills.map((entry) => entry.percent)).toEqual([40])
  })
})
