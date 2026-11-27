import { beforeEach, describe, expect, it, vi } from 'vitest'
import { enqueueTask, fetchRunQueue, runTaskInQueue } from '@/features/queue/api/runQueue.api'
import { EMPTY_RUN_QUEUE } from '@/features/queue/model/runQueue'
import { asTaskId } from '@/model/branded'

const client = vi.hoisted(() => vi.fn())

vi.mock('@/api/client', () => ({
  apiClient: client,
  request: <T>(operation: () => Promise<T>) => operation()
}))

const taskId = asTaskId('aaaaaaaa-0000-4000-8000-000000000001')
const item = {
  id: 'bbbbbbbb-0000-4000-8000-000000000001',
  taskId,
  taskTitle: 'Report',
  taskCode: 'RV-1',
  projectLabel: 'rekall',
  anchor: 'task:RV-1',
  position: 0,
  state: 'QUEUED',
  detail: null,
  startedAt: null,
  finishedAt: null
}

describe('the run queue API', () => {
  beforeEach(() => {
    client.mockReset()
  })

  it('sends confirmed Codex options for both Add and Run and reads the saved options', async () => {
    const codexOptions = { model: 'example/codex', effort: 'high', skipPermissions: true }
    client.mockResolvedValue({ ...EMPTY_RUN_QUEUE, items: [{ ...item, provider: 'CODEX', codexOptions }] })

    const launch = { taskId, provider: 'CODEX', codexOptions } as const
    const added = await enqueueTask(launch)
    const run = await runTaskInQueue(launch)

    const request = { method: 'POST', retry: 0, body: launch }
    expect(client).toHaveBeenNthCalledWith(1, '/api/run-queue/items', request)
    expect(client).toHaveBeenNthCalledWith(2, '/api/run-queue/runs', request)
    expect(added.items[0]).toMatchObject({ provider: 'CODEX', codexOptions })
    expect(run.items[0]).toMatchObject({ provider: 'CODEX', codexOptions })
  })

  it.each([
    { model: 'sonnet', effort: 'high' },
    { model: null, effort: null }
  ])('round-trips Claude Code choices $model and $effort in both write responses', async (claudeOptions) => {
    const launch = { taskId, provider: 'CLAUDE', claudeOptions } as const
    client.mockResolvedValue({ ...EMPTY_RUN_QUEUE, items: [{ ...item, provider: 'CLAUDE', claudeOptions }] })

    const added = await enqueueTask(launch)
    const run = await runTaskInQueue(launch)

    const request = { method: 'POST', retry: 0, body: launch }
    expect(client).toHaveBeenNthCalledWith(1, '/api/run-queue/items', request)
    expect(client).toHaveBeenNthCalledWith(2, '/api/run-queue/runs', request)
    expect(added.items[0]?.claudeOptions).toEqual(claudeOptions)
    expect(run.items[0]?.claudeOptions).toEqual(claudeOptions)
  })

  it('reads a queue response without provider fields as Claude Code', async () => {
    client.mockResolvedValue({ ...EMPTY_RUN_QUEUE, items: [item] })

    const queue = await fetchRunQueue()

    expect(queue.items[0]).toMatchObject({ provider: 'CLAUDE', codexOptions: null, claudeOptions: null })
  })
})
