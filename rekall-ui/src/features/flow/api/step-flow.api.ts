import { apiClient, request } from '../../../api/client'
import { TaskStepListSchema } from '../../catalog/api/schemas/catalog.schema'
import { isFlowEnd, type RouteTarget } from '@/features/flow/lib/route-target'
import type { TaskStep } from '@/features/catalog/model/catalog'
import type { StepLinkId, TaskId, TaskStepId } from '@/model/branded'

/** A route as the console edits it: on create a target and an outcome are sent, on edit either may be left out. */
export interface StepLinkPatch {
  toStepId?: TaskStepId
  endsFlow?: boolean
  outcome?: string
}

/** A step dragged on the canvas, in canvas pixels. */
export interface FlowPoint {
  readonly x: number
  readonly y: number
}

/** Every call answers the task's whole checklist, since a route changes how each step reads. */
export async function createStepLink(
  fromStepId: TaskStepId,
  target: RouteTarget,
  outcome: string
): Promise<TaskStep[]> {
  const body = isFlowEnd(target) ? { endsFlow: true, outcome } : { toStepId: target, outcome }
  return request(async () =>
    TaskStepListSchema.parse(await apiClient(`/api/steps/${fromStepId}/links`, { method: 'POST', body }))
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

/** Remembers where a step was dragged to; null hands it back to the automatic layout. */
export async function placeStepInFlow(id: TaskStepId, at: FlowPoint | null): Promise<TaskStep[]> {
  return request(async () =>
    TaskStepListSchema.parse(
      await apiClient(`/api/steps/${id}/flow-place`, {
        method: 'PATCH',
        body: { x: at?.x ?? null, y: at?.y ?? null }
      })
    )
  )
}

/** Caps how many times a flow may enter the step; null lifts the cap. */
export async function limitStepVisits(id: TaskStepId, maxVisits: number | null): Promise<TaskStep[]> {
  return request(async () =>
    TaskStepListSchema.parse(
      await apiClient(`/api/steps/${id}/max-visits`, { method: 'PATCH', body: { maxVisits } })
    )
  )
}

/** Hands every step of the task back to the automatic layout. */
export async function clearFlowPlaces(taskId: TaskId): Promise<TaskStep[]> {
  return request(async () =>
    TaskStepListSchema.parse(await apiClient(`/api/tasks/${taskId}/flow-places`, { method: 'DELETE' }))
  )
}
