import { apiClient, request } from './client'
import { CommitReferenceSchema } from './schemas/commitReference.schema'
import type { CommitReference } from '@/model/commitReference'
import type { TaskId, TaskStepId } from '@/model/branded'

export async function recordLatestCommit(
  taskId: TaskId,
  stepId: TaskStepId | null
): Promise<CommitReference> {
  return request(async () =>
    CommitReferenceSchema.parse(
      await apiClient(`/api/tasks/${taskId}/commit-references/latest`, {
        method: 'POST',
        body: { stepId: stepId ?? null }
      })
    )
  )
}
