import { apiClient, request } from '../../../api/client'
import { ContextSizeSchema } from './schemas/context.schema'
import type { TaskId } from '@/model/branded'
import type { ContextSize } from '@/features/context/model/context'

export async function fetchContextSize(taskId: TaskId): Promise<ContextSize> {
  return request(async () => ContextSizeSchema.parse(await apiClient(`/api/tasks/${taskId}/context-size`)))
}
