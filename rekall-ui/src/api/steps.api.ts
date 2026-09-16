import { apiClient, request } from './client'
import { TaskStepListSchema, TaskStepSchema } from './schemas/catalog.schema'
import type { TaskStep } from '@/model/catalog'
import type { TaskId, TaskStepId } from '@/model/branded'

export interface TaskStepPatch {
  title?: string
  bodyMarkdown?: string
  done?: boolean
  draft?: boolean
}

export async function fetchSteps(): Promise<TaskStep[]> {
  return request(async () => TaskStepListSchema.parse(await apiClient('/api/steps')))
}

export async function createStep(
  taskId: TaskId,
  title: string,
  bodyMarkdown?: string
): Promise<TaskStep> {
  return request(async () =>
    TaskStepSchema.parse(
      await apiClient(`/api/tasks/${taskId}/steps`, {
        method: 'POST',
        body: { title, bodyMarkdown: bodyMarkdown ?? null }
      })
    )
  )
}

export async function patchStep(id: TaskStepId, patch: TaskStepPatch): Promise<TaskStep> {
  return request(async () =>
    TaskStepSchema.parse(await apiClient(`/api/steps/${id}`, { method: 'PATCH', body: patch }))
  )
}

export async function moveStep(id: TaskStepId, position: number): Promise<TaskStep[]> {
  return request(async () =>
    TaskStepListSchema.parse(
      await apiClient(`/api/steps/${id}/move`, { method: 'POST', body: { position } })
    )
  )
}

export async function deleteStep(id: TaskStepId): Promise<void> {
  await request(() => apiClient(`/api/steps/${id}`, { method: 'DELETE' }))
}
