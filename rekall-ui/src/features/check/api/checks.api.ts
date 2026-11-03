import { apiClient, request } from '../../../api/client'
import { CheckResultListSchema, DoneCheckListSchema, DoneCheckSchema } from './schemas/check.schema'
import type { CheckInput, CheckResult, DoneCheck } from '@/features/check/model/check'
import type { DoneCheckId, ProjectId, TaskId, TaskStepId } from '@/model/branded'

export async function fetchChecks(projectId: ProjectId): Promise<DoneCheck[]> {
  return request(async () => DoneCheckListSchema.parse(await apiClient(`/api/projects/${projectId}/checks`)))
}

export async function createProjectCheck(projectId: ProjectId, input: CheckInput): Promise<DoneCheck> {
  return request(async () =>
    DoneCheckSchema.parse(await apiClient(`/api/projects/${projectId}/checks`, { method: 'POST', body: input }))
  )
}

export async function createStepCheck(stepId: TaskStepId, input: CheckInput): Promise<DoneCheck> {
  return request(async () =>
    DoneCheckSchema.parse(await apiClient(`/api/steps/${stepId}/checks`, { method: 'POST', body: input }))
  )
}

export async function updateCheck(id: DoneCheckId, input: CheckInput): Promise<DoneCheck> {
  return request(async () =>
    DoneCheckSchema.parse(await apiClient(`/api/checks/${id}`, { method: 'PUT', body: input }))
  )
}

export async function reorderProjectChecks(projectId: ProjectId, checkIds: readonly DoneCheckId[]): Promise<DoneCheck[]> {
  return request(async () =>
    DoneCheckListSchema.parse(
      await apiClient(`/api/projects/${projectId}/checks/order`, { method: 'PUT', body: { checkIds } })
    )
  )
}

export async function reorderStepChecks(stepId: TaskStepId, checkIds: readonly DoneCheckId[]): Promise<DoneCheck[]> {
  return request(async () =>
    DoneCheckListSchema.parse(await apiClient(`/api/steps/${stepId}/checks/order`, { method: 'PUT', body: { checkIds } }))
  )
}

export async function deleteCheck(id: DoneCheckId): Promise<void> {
  await request(() => apiClient(`/api/checks/${id}`, { method: 'DELETE' }))
}

export async function fetchTaskCheckResults(taskId: TaskId): Promise<CheckResult[]> {
  return request(async () => CheckResultListSchema.parse(await apiClient(`/api/tasks/${taskId}/check-results`)))
}
