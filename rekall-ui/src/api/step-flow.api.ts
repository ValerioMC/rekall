import { apiClient, request } from './client'
import { TaskStepListSchema } from './schemas/catalog.schema'
import type { TaskStep } from '@/model/catalog'
import type { StepLinkId, TaskId, TaskStepId } from '@/model/branded'

/** A route as the console edits it: on create both are sent, on edit either may be left out. */
export interface StepLinkPatch {
  toStepId?: TaskStepId
  outcome?: string
}

/** Every call answers the task's whole checklist, since a route changes how each step reads. */
export async function createStepLink(
  fromStepId: TaskStepId,
  toStepId: TaskStepId,
  outcome: string
): Promise<TaskStep[]> {
  return request(async () =>
    TaskStepListSchema.parse(
      await apiClient(`/api/steps/${fromStepId}/links`, { method: 'POST', body: { toStepId, outcome } })
    )
  )
}

export async function patchStepLink(id: StepLinkId, patch: StepLinkPatch): Promise<TaskStep[]> {
  return request(async () =>
    TaskStepListSchema.parse(await apiClient(`/api/step-links/${id}`, { method: 'PATCH', body: patch }))
  )
}

export async function deleteStepLink(id: StepLinkId): Promise<TaskStep[]> {
  return request(async () =>
    TaskStepListSchema.parse(await apiClient(`/api/step-links/${id}`, { method: 'DELETE' }))
  )
}

/** Forgets where the flow went; the steps keep their states. */
export async function resetStepVisits(taskId: TaskId): Promise<TaskStep[]> {
  return request(async () =>
    TaskStepListSchema.parse(await apiClient(`/api/tasks/${taskId}/visits`, { method: 'DELETE' }))
  )
}
