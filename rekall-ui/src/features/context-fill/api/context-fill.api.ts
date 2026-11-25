import { apiClient, request } from '../../../api/client'
import { ContextFillListSchema } from './schemas/context-fill.schema'
import type { ContextFill } from '@/features/context-fill/model/context-fill'
import type { TaskId } from '@/model/branded'

export async function fetchContextFills(taskId: TaskId): Promise<ContextFill[]> {
  return request(async () =>
    ContextFillListSchema.parse(await apiClient(`/api/context-fill?taskId=${encodeURIComponent(taskId)}`))
  )
}
