import { z } from 'zod'
import { asTaskId } from '@/model/branded'

export const TaskDependencySchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  anchor: z.string(),
  title: z.string(),
  met: z.boolean(),
  awaitingReview: z.boolean()
})

export const TaskDependencyListSchema = z.array(TaskDependencySchema)
