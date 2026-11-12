import { apiClient, request } from '../../../api/client'
import { OptionalTaskBranchSchema } from './schemas/taskBranch.schema'
import type { TaskBranch } from '@/features/worktree/model/taskBranch'
import type { TaskId } from '@/model/branded'

/** Makes the task's branch and worktree when its project keeps tasks on branches; null when it does not. */
export async function startTaskBranch(taskId: TaskId): Promise<TaskBranch | null> {
  return request(async () => OptionalTaskBranchSchema.parse(await apiClient(`/api/tasks/${taskId}/branch`, { method: 'POST' })))
}

export async function fetchTaskBranch(taskId: TaskId): Promise<TaskBranch | null> {
  return request(async () => OptionalTaskBranchSchema.parse(await apiClient(`/api/tasks/${taskId}/branch`)))
}
