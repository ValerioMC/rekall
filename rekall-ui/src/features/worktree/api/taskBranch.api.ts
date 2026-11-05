import { apiClient, request } from '../../../api/client'
import { OptionalTaskBranchSchema } from './schemas/taskBranch.schema'
import type { TaskBranch } from '@/features/worktree/model/taskBranch'
import type { TaskId } from '@/model/branded'

export async function fetchTaskBranch(taskId: TaskId): Promise<TaskBranch | null> {
  return request(async () => OptionalTaskBranchSchema.parse(await apiClient(`/api/tasks/${taskId}/branch`)))
}
