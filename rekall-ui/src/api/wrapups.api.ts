import { apiClient, request } from './client'
import { WrapupListSchema, WrapupSchema } from './schemas/catalog.schema'
import type { Wrapup } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

/**
 * A wrapup is addressed by its task, because that is its identity: one task, one wrapup. There
 * is no collection to post into and no id to remember, so a write is a PUT at the task's
 * address and it either creates or replaces.
 */
export async function fetchWrapups(): Promise<Wrapup[]> {
  return request(async () => WrapupListSchema.parse(await apiClient('/api/wrapups')))
}

/** Sends the whole text. A wrapup is replaced, never amended, so there is no partial form. */
export async function saveWrapup(taskId: TaskId, bodyMarkdown: string): Promise<Wrapup> {
  return request(async () =>
    WrapupSchema.parse(
      await apiClient(`/api/tasks/${taskId}/wrapup`, { method: 'PUT', body: { bodyMarkdown } })
    )
  )
}

/** Removes the wrapup and nothing else. The task and its notes are untouched. */
export async function deleteWrapup(taskId: TaskId): Promise<void> {
  await request(() => apiClient(`/api/tasks/${taskId}/wrapup`, { method: 'DELETE' }))
}
