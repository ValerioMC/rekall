import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/tokens/api/token-usage.api'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import type { TaskId, TaskStepId } from '@/model/branded'
import type { TokenDay, TokenTally } from '@/features/tokens/model/token-usage'

vi.mock('@/features/tokens/api/token-usage.api')

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

function day(taskId: string, on: string, outputTokens: number): TokenDay {
  return { taskId: taskId as TaskId, day: on, inputTokens: 0, outputTokens, cacheCreationTokens: 0, cacheReadTokens: 0 }
}

describe('the token usage store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('reads the tallies and answers per step and per task', async () => {
    vi.mocked(api.fetchTokenTallies).mockResolvedValue([tally('t1', null, 5), tally('t1', 's1', 7)])
    vi.mocked(api.fetchTokenDays).mockResolvedValue([])
    const store = useTokenUsageStore()

    await store.load()

    expect(store.ofStep('s1' as TaskStepId).generated).toBe(7)
    expect(store.ofTask('t1' as TaskId).generated).toBe(12)
  })

  it('takes a task’s tallies from a stream event in place of the ones it held', async () => {
    vi.mocked(api.fetchTokenTallies).mockResolvedValue([tally('t1', null, 5), tally('t2', null, 3)])
    vi.mocked(api.fetchTokenDays).mockResolvedValue([day('t1', '2026-10-01', 5), day('t2', '2026-10-01', 3)])
    const store = useTokenUsageStore()
    await store.load()

    store.apply({ taskId: 't1' as TaskId, tallies: [tally('t1', 's1', 40)], days: [day('t1', '2026-10-02', 40)] })

    expect(store.ofTask('t1' as TaskId).generated).toBe(40)
    expect(store.ofTask('t2' as TaskId).generated).toBe(3)
    expect(store.days.map((entry) => [entry.taskId, entry.day])).toEqual([
      ['t2', '2026-10-01'],
      ['t1', '2026-10-02']
    ])
  })
})
