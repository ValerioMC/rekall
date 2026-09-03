import { apiClient, request } from './client'
import { TaskStepListSchema, TaskStepSchema } from './schemas/catalog.schema'
import type { TaskStep } from '@/model/catalog'
import type { TaskId, TaskStepId } from '@/model/branded'

/** Every field optional, because ticking a box must not resend a detail the row never loaded. */
export interface TaskStepPatch {
  title?: string
  bodyMarkdown?: string
  done?: boolean
}

/** Every step, loaded whole for the same reason the wrapups are: no per-task round trip. */
export async function fetchSteps(): Promise<TaskStep[]> {
  return request(async () => TaskStepListSchema.parse(await apiClient('/api/steps')))
}

/** Appends to the end of the task's checklist. Where it lands is the server's to decide. */
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

/**
 * Moves one step and answers with its task's whole checklist.
 *
 * A move renumbers everything it displaced, so the response is the list rather than the row:
 * patching the moved step alone would leave the client holding positions that are no longer
 * true.
 */
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
