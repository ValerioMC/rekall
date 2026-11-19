import { apiClient, request } from '../../../api/client'
import { TaskChangesSchema } from './schemas/taskChanges.schema'
import type { TaskChanges } from '@/features/commits/model/taskChanges'
import type { TaskId } from '@/model/branded'

/** Every logged commit's patch for the task, oldest first, and the working tree of its folder. */
export async function fetchTaskChanges(taskId: TaskId): Promise<TaskChanges> {
  return request(async () => TaskChangesSchema.parse(await apiClient(`/api/tasks/${taskId}/changes`)))
}
