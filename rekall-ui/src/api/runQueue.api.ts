import { apiClient, request } from './client'
import { RunQueueSchema } from './schemas/runQueue.schema'
import type { RunQueue, RunQueueItemId, RunQueueSettings } from '@/model/runQueue'
import type { TaskId } from '@/model/branded'

/** Every call answers with the whole queue, the same shape the `run-queue` SSE event carries. */
async function queueCall(path: string, options: Parameters<typeof apiClient>[1] = {}): Promise<RunQueue> {
  return request(async () => RunQueueSchema.parse(await apiClient(`/api/run-queue${path}`, options)))
}

export function fetchRunQueue(): Promise<RunQueue> {
  return queueCall('')
}

export function saveRunQueueSettings(settings: RunQueueSettings): Promise<RunQueue> {
  return queueCall('/settings', { method: 'PUT', body: settings })
}

export function enqueueTask(taskId: TaskId): Promise<RunQueue> {
  return queueCall('/items', { method: 'POST', body: { taskId } })
}

export function dequeueItem(itemId: RunQueueItemId): Promise<RunQueue> {
  return queueCall(`/items/${itemId}`, { method: 'DELETE' })
}

/** `index` counts among the items still waiting, not the whole list. */
export function moveQueueItem(itemId: RunQueueItemId, index: number): Promise<RunQueue> {
  return queueCall(`/items/${itemId}/position`, { method: 'PUT', body: { index } })
}

export function clearSettledItems(): Promise<RunQueue> {
  return queueCall('/clear', { method: 'POST' })
}

/** `startAt` null starts at once. */
export function startRunQueue(startAt: string | null): Promise<RunQueue> {
  return queueCall('/start', { method: 'POST', body: { startAt } })
}

export function stopRunQueue(): Promise<RunQueue> {
  return queueCall('/stop', { method: 'POST' })
}
