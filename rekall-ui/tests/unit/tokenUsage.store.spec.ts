import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/api/token-usage.api'
import { useTokenUsageStore } from '@/stores/tokenUsage.store'
import type { TaskId, TaskStepId } from '@/model/branded'
import type { TokenTally } from '@/model/token-usage'

vi.mock('@/api/token-usage.api')

function tally(taskId: string, stepId: string | null, outputTokens: number): TokenTally {
  return {
    taskId: taskId as TaskId,
    stepId: stepId as TaskStepId | null,
    inputTokens: 0,
    outputTokens,
    cacheCreationTokens: 0,
    cacheReadTokens: 0
  }
}

describe('the token usage store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('reads the tallies and answers per step and per task', async () => {
    vi.mocked(api.fetchTokenTallies).mockResolvedValue([tally('t1', null, 5), tally('t1', 's1', 7)])
    const store = useTokenUsageStore()

    await store.load()

    expect(store.ofStep('s1' as TaskStepId).generated).toBe(7)
    expect(store.ofTask('t1' as TaskId).generated).toBe(12)
  })

  it('takes a task’s tallies from a stream event in place of the ones it held', async () => {
    vi.mocked(api.fetchTokenTallies).mockResolvedValue([tally('t1', null, 5), tally('t2', null, 3)])
    const store = useTokenUsageStore()
    await store.load()

    store.apply({ taskId: 't1' as TaskId, tallies: [tally('t1', 's1', 40)] })

    expect(store.ofTask('t1' as TaskId).generated).toBe(40)
    expect(store.ofTask('t2' as TaskId).generated).toBe(3)
  })
})
