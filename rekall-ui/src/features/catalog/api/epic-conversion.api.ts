import { z } from 'zod'
import { apiClient, request } from '../../../api/client'
import { TaskSchema } from './schemas/catalog.schema'
import type { EpicConversion } from '@/features/catalog/model/epic-conversion'
import type { Task } from '@/features/catalog/model/catalog'
import { asTaskId, type TaskId } from '@/model/branded'

const EpicConversionSchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  steps: z.number().int(),
  stepsDone: z.number().int(),
  routes: z.number().int(),
  hasWrapup: z.boolean(),
  commits: z.number().int(),
  deviations: z.number().int(),
  waitsFor: z.number().int(),
  neededBy: z.number().int(),
  skills: z.number().int(),
  hasPullRequest: z.boolean(),
  epicCode: z.string().nullable(),
  epicTitle: z.string().nullable(),
  queued: z.boolean()
})

/** What turning the task into an epic would delete. */
export async function fetchEpicConversion(id: TaskId): Promise<EpicConversion> {
  return request(async () => EpicConversionSchema.parse(await apiClient(`/api/tasks/${id}/epic-conversion`)))
}

export async function convertTaskToEpic(id: TaskId): Promise<Task> {
  return request(async () => TaskSchema.parse(await apiClient(`/api/tasks/${id}/epic-conversion`, { method: 'POST' })))
}
