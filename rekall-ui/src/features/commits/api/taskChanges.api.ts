import { apiClient, request } from '../../../api/client'
import { ChangeFileBatchSchema, TaskChangesIndexSchema } from './schemas/taskChanges.schema'
import { PROJECT_REPOSITORY_ID, type ChangeScope } from '@/features/commits/model/changeScope'
import type { ChangeFileHunks, ChangeFileKey, TaskChangesIndex } from '@/features/commits/model/taskChanges'
import type { TaskId } from '@/model/branded'

export async function fetchTaskChangesIndex(taskId: TaskId): Promise<TaskChangesIndex> {
  return request(async () => TaskChangesIndexSchema.parse(await apiClient(`/api/tasks/${taskId}/changes/index`)))
}

function scopeRequest(scope: ChangeScope) {
  if (scope.kind !== 'repository' && scope.kind !== 'working-tree') return scope
  return { kind: scope.kind, componentId: scope.repositoryId === PROJECT_REPOSITORY_ID ? null : scope.repositoryId }
}

export async function fetchTaskChangeFiles(
  taskId: TaskId,
  snapshotId: string,
  scope: ChangeScope,
  files: readonly ChangeFileKey[]
): Promise<readonly ChangeFileHunks[]> {
  return request(async () => ChangeFileBatchSchema.parse(await apiClient(`/api/tasks/${taskId}/changes/files`, {
    method: 'POST',
    body: { snapshotId, scope: scopeRequest(scope), files }
  })))
}
