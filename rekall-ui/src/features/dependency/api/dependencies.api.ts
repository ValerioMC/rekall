import { apiClient, request } from '../../../api/client'
import { TaskDependencyListSchema } from './schemas/dependency.schema'
import type { TaskDependency } from '@/features/dependency/model/dependency'
import type { TaskId } from '@/model/branded'

export async function fetchDependencies(taskId: TaskId): Promise<TaskDependency[]> {
  return request(async () => TaskDependencyListSchema.parse(await apiClient(`/api/tasks/${taskId}/dependencies`)))
}

export async function addDependency(taskId: TaskId, dependsOnTaskId: TaskId): Promise<TaskDependency[]> {
  return request(async () =>
    TaskDependencyListSchema.parse(
      await apiClient(`/api/tasks/${taskId}/dependencies`, { method: 'POST', body: { dependsOnTaskId } })
    )
  )
}

export async function removeDependency(taskId: TaskId, dependsOnTaskId: TaskId): Promise<TaskDependency[]> {
  return request(async () =>
    TaskDependencyListSchema.parse(
      await apiClient(`/api/tasks/${taskId}/dependencies/${dependsOnTaskId}`, { method: 'DELETE' })
    )
  )
}
