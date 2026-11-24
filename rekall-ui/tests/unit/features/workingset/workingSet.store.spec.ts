import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/workingset/api/working-set.api'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import { useToastStore } from '@/stores/toast.store'
import type { ProjectId, TaskId } from '@/model/branded'

vi.mock('@/features/workingset/api/working-set.api')

const T1 = 't1' as TaskId
const T2 = 't2' as TaskId
const VEGA = 'vega' as ProjectId

describe('the working set store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('reads the set from the server', async () => {
    vi.mocked(api.fetchWorkingSet).mockResolvedValue({ tasks: [{ taskId: T1, projectId: VEGA }], places: [{ projectId: VEGA, taskId: T1 }] })
    const store = useWorkingSetStore()

    await store.load()

    expect(store.isOpen(T1)).toBe(true)
    expect(store.placeOf(VEGA)).toEqual({ projectId: VEGA, taskId: T1 })
  })

  it('shows an opened task at once and does not ask the server again for one already open', async () => {
    vi.mocked(api.openTask).mockResolvedValue({ tasks: [], places: [] })
    const store = useWorkingSetStore()

    await store.open(T1, VEGA)
    await store.open(T1, VEGA)

    expect(store.isOpen(T1)).toBe(true)
    expect(api.openTask).toHaveBeenCalledTimes(1)
  })

  it('takes an opened task back and says so when the server refuses it', async () => {
    vi.mocked(api.openTask).mockRejectedValue(new Error('No Task with id t1'))
    const store = useWorkingSetStore()

    await store.open(T1, VEGA)

    expect(store.isOpen(T1)).toBe(false)
    expect(useToastStore().toasts).toHaveLength(1)
  })

  it('closes a task and answers where its project lands, remembering the landing when the project had remembered the closed task', async () => {
    vi.mocked(api.fetchWorkingSet).mockResolvedValue({
      tasks: [{ taskId: T1, projectId: VEGA }, { taskId: T2, projectId: VEGA }],
      places: [{ projectId: VEGA, taskId: T1 }]
    })
    vi.mocked(api.closeTask).mockResolvedValue({ projectId: VEGA, taskId: T2 })
    const store = useWorkingSetStore()
    await store.load()

    const landing = await store.close(T1)

    expect(landing).toEqual({ projectId: VEGA, taskId: T2 })
    expect(store.isOpen(T1)).toBe(false)
    expect(store.placeOf(VEGA).taskId).toBe(T2)
  })

  it('puts a task back when closing it is refused', async () => {
    vi.mocked(api.fetchWorkingSet).mockResolvedValue({ tasks: [{ taskId: T1, projectId: VEGA }], places: [] })
    vi.mocked(api.closeTask).mockRejectedValue(new Error('boom'))
    const store = useWorkingSetStore()
    await store.load()

    const landing = await store.close(T1)

    expect(landing).toBeNull()
    expect(store.isOpen(T1)).toBe(true)
  })

  it('remembers a place at once and goes back to the one before when the server refuses', async () => {
    vi.mocked(api.rememberPlace).mockRejectedValue(new Error('boom'))
    const store = useWorkingSetStore()

    await store.remember({ projectId: VEGA, taskId: T1 })

    expect(store.placeOf(VEGA).taskId).toBeNull()
    expect(useToastStore().toasts).toHaveLength(1)
  })
})
