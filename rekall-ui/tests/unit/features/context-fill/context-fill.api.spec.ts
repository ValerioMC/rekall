import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fetchContextFills } from '@/features/context-fill/api/context-fill.api'
import { ContextFillStreamEventSchema } from '@/features/context-fill/api/schemas/context-fill.schema'
import { STEP, TASK } from './fixtures'

const client = vi.hoisted(() => vi.fn())

vi.mock('@/api/client', () => ({
  apiClient: client,
  request: <T>(operation: () => Promise<T>) => operation()
}))

const TASK_ROW = { stepId: null, usedTokens: 84000, windowTokens: 200000, percent: 42, recordedAt: '2026-11-25T08:00:00Z' }
const STEP_ROW = { ...TASK_ROW, stepId: STEP, percent: 21 }

describe('the context fill api', () => {
  beforeEach(() => client.mockReset())

  it('asks for the fills of a task and reads the list the server sends', async () => {
    client.mockResolvedValue([TASK_ROW, STEP_ROW])

    const fills = await fetchContextFills(TASK)

    expect(client).toHaveBeenCalledWith(`/api/context-fill?taskId=${TASK}`)
    expect(fills.map((fill) => [fill.stepId, fill.percent])).toEqual([
      [null, 42],
      [STEP, 21]
    ])
  })

  it('refuses a row with a percent out of range or a step that is not an id', async () => {
    client.mockResolvedValue([{ ...TASK_ROW, percent: 120 }])
    await expect(fetchContextFills(TASK)).rejects.toThrow()

    client.mockResolvedValue([{ ...TASK_ROW, stepId: 'not-an-id' }])
    await expect(fetchContextFills(TASK)).rejects.toThrow()
  })

  it('reads the frame the stream sends: a task and its whole list', () => {
    const parsed = ContextFillStreamEventSchema.parse({ taskId: TASK, fills: [TASK_ROW, STEP_ROW] })

    expect(parsed.taskId).toBe(TASK)
    expect(parsed.fills).toHaveLength(2)
  })
})
