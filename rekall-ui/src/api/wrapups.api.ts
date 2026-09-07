import { apiClient, request } from './client'
import { WrapupListSchema, WrapupSchema } from './schemas/catalog.schema'
import type { Wrapup } from '@/model/catalog'
import type { TaskId } from '@/model/branded'

export async function fetchWrapups(): Promise<Wrapup[]> {
  return request(async () => WrapupListSchema.parse(await apiClient('/api/wrapups')))
}

export async function saveWrapup(taskId: TaskId, bodyMarkdown: string): Promise<Wrapup> {
  return request(async () =>
    WrapupSchema.parse(
      await apiClient(`/api/tasks/${taskId}/wrapup`, { method: 'PUT', body: { bodyMarkdown } })
    )
  )
}

export async function deleteWrapup(taskId: TaskId): Promise<void> {
  await request(() => apiClient(`/api/tasks/${taskId}/wrapup`, { method: 'DELETE' }))
}
