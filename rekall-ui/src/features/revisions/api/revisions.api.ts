import { apiClient, request } from '../../../api/client'
import { RestoredRevisionSchema, TaskRevisionListSchema } from './schemas/revision.schema'
import type { TaskId } from '@/model/branded'
import type { RestoredRevision, RevisionKind, TaskRevision } from '@/features/revisions/model/revision'

export async function fetchRevisions(taskId: TaskId, kind: RevisionKind): Promise<TaskRevision[]> {
  return request(async () =>
    TaskRevisionListSchema.parse(await apiClient(`/api/tasks/${taskId}/revisions`, { query: { kind } }))
  )
}

export async function restoreRevision(taskId: TaskId, revisionId: string): Promise<RestoredRevision> {
  return request(async () =>
    RestoredRevisionSchema.parse(
      await apiClient(`/api/tasks/${taskId}/revisions/${revisionId}/restore`, { method: 'POST' })
    )
  )
}
