import { apiClient, request } from '@/api/client'
import { OpenPlaceSchema, WorkingSetSchema } from './schemas/working-set.schema'
import type { OpenPlace, WorkingSet } from '@/features/workingset/model/working-set'
import type { ProjectId, TaskId } from '@/model/branded'

export async function fetchWorkingSet(): Promise<WorkingSet> {
  return request(async () => WorkingSetSchema.parse(await apiClient('/api/working-set')))
}

export async function openTask(taskId: TaskId): Promise<WorkingSet> {
  return request(async () => WorkingSetSchema.parse(await apiClient(`/api/working-set/tasks/${taskId}`, { method: 'PUT' })))
}

/** Closes the task; the answer is where its project lands. */
export async function closeTask(taskId: TaskId): Promise<OpenPlace> {
  return request(async () => OpenPlaceSchema.parse(await apiClient(`/api/working-set/tasks/${taskId}`, { method: 'DELETE' })))
}

export async function rememberPlace(projectId: ProjectId, taskId: TaskId | null): Promise<OpenPlace> {
  return request(async () =>
    OpenPlaceSchema.parse(await apiClient(`/api/working-set/places/${projectId}`, { method: 'PUT', body: { taskId } }))
  )
}
