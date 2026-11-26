import { beforeEach, describe, expect, it, vi } from 'vitest'
import { setCodexBypass, setCodexEffort, setCodexModel } from '@/common/config/session-launch'
import { enqueueTask, runTaskInQueue } from '@/features/queue/api/runQueue.api'
import { EMPTY_RUN_QUEUE } from '@/features/queue/model/runQueue'
import { asTaskId } from '@/model/branded'
import { installStorage } from '../../support/memoryStorage'

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
    installStorage()
    client.mockReset()
  })

  it('sends the Codex preferences for both Add and Run and reads the saved options', async () => {
    setCodexModel('example/codex')
    setCodexEffort('high')
    setCodexBypass(true)
    const codexOptions = { model: 'example/codex', effort: 'high', skipPermissions: true }
    client.mockResolvedValue({ ...EMPTY_RUN_QUEUE, items: [{ ...item, provider: 'CODEX', codexOptions }] })

    const added = await enqueueTask(taskId, 'CODEX')
    const run = await runTaskInQueue(taskId, 'CODEX')

    const request = { method: 'POST', retry: 0, body: { taskId, provider: 'CODEX', codexOptions } }
    expect(client).toHaveBeenNthCalledWith(1, '/api/run-queue/items', request)
    expect(client).toHaveBeenNthCalledWith(2, '/api/run-queue/runs', request)
    expect(added.items[0]).toMatchObject({ provider: 'CODEX', codexOptions })
    expect(run.items[0]).toMatchObject({ provider: 'CODEX', codexOptions })
  })

  it('reads a queue response without provider fields as Claude Code', async () => {
    client.mockResolvedValue({ ...EMPTY_RUN_QUEUE, items: [item] })

    const queue = await enqueueTask(taskId, 'CLAUDE')

    expect(client).toHaveBeenCalledWith('/api/run-queue/items', { method: 'POST', retry: 0, body: { taskId, provider: 'CLAUDE' } })
    expect(queue.items[0]).toMatchObject({ provider: 'CLAUDE', codexOptions: null })
  })
})
