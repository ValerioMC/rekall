import { z } from 'zod'
import { apiClient, request } from './client'
import { CommitReferenceDiffSchema, CommitReferenceSchema } from './schemas/commitReference.schema'
import type { CommitReference } from '@/model/commitReference'
import type { TaskId, TaskStepId } from '@/model/branded'

export async function fetchCommitReferences(): Promise<CommitReference[]> {
  return request(async () =>
    z.array(CommitReferenceSchema).parse(await apiClient('/api/commit-references'))
  )
}

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

/** Fetched on demand, not with the list: most rows are never opened. */
export async function fetchCommitReferenceDiff(id: string): Promise<string | null> {
  return request(async () =>
    CommitReferenceDiffSchema.parse(await apiClient(`/api/commit-references/${id}/diff`)).diff
  )
}
