import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/deviations/api/deviations.api'
import { useDeviationsStore } from '@/features/deviations/store/deviations.store'
import { deviation, taskId } from './fixtures'
import type { TaskId } from '@/model/branded'

vi.mock('@/features/deviations/api/deviations.api')

describe('the deviations store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('keeps the deviations of every task when several load at once', async () => {
    const other = 't2' as TaskId
    const first = Promise.withResolvers<ReturnType<typeof deviation>[]>()
    vi.mocked(api.fetchDeviations).mockImplementation((id) => (id === taskId ? first.promise : Promise.resolve([deviation(2, { taskId: other })])))
    const store = useDeviationsStore()

    const slow = store.load(taskId)
    await store.load(other)
    first.resolve([deviation(1)])
    await slow

    expect(store.of(taskId).map((entry) => entry.stepNumber)).toEqual([1])
    expect(store.of(other).map((entry) => entry.stepNumber)).toEqual([2])
  })
})
