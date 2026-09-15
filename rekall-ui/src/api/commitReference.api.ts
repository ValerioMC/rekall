import { z } from 'zod'
import { apiClient, request } from './client'
import {
  CommitReferenceDiffSchema,
  CommitReferenceSchema,
  RecentCommitSchema
} from './schemas/commitReference.schema'
import type { CommitReference, RecentCommit } from '@/model/commitReference'
import type { TaskId, TaskStepId } from '@/model/branded'

export async function fetchCommitReferences(): Promise<CommitReference[]> {
  return request(async () => z.array(CommitReferenceSchema).parse(await apiClient('/api/commit-references')))
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

/** The newest commits of the task's project folder, for picking one by hand. */
export async function fetchRecentCommits(taskId: TaskId): Promise<RecentCommit[]> {
  return request(async () =>
    z.array(RecentCommitSchema).parse(await apiClient(`/api/tasks/${taskId}/recent-commits`))
  )
}

/** A commit chosen from the recent log or pasted as a hash, abbreviated or full. */
export async function recordCommit(
  taskId: TaskId,
  stepId: TaskStepId | null,
  commitHash: string
): Promise<CommitReference> {
  return request(async () =>
    CommitReferenceSchema.parse(
      await apiClient(`/api/tasks/${taskId}/commit-references`, {
        method: 'POST',
        body: { stepId: stepId ?? null, commitHash }
      })
    )
  )
}

/** Fetched on demand, not with the list: most rows are never opened. */
export async function fetchCommitReferenceDiff(id: string): Promise<string | null> {
  return request(
    async () => CommitReferenceDiffSchema.parse(await apiClient(`/api/commit-references/${id}/diff`)).diff
  )
}

export async function deleteCommitReference(id: string): Promise<void> {
  await request(() => apiClient(`/api/commit-references/${id}`, { method: 'DELETE' }))
}
