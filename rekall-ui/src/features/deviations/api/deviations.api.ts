import { apiClient, request } from '../../../api/client'
import { TaskDeviationListSchema } from './schemas/deviation.schema'
import type { TaskDeviation } from '@/features/deviations/model/deviation'
import type { TaskId } from '@/model/branded'

export async function fetchDeviations(taskId: TaskId): Promise<TaskDeviation[]> {
  return request(async () => TaskDeviationListSchema.parse(await apiClient(`/api/tasks/${taskId}/deviations`)))
}
