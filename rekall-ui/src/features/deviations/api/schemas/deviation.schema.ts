import { z } from 'zod'
import { asTaskId } from '@/model/branded'

export const TaskDeviationSchema = z.object({
  id: z.string().uuid(),
  taskId: z.string().uuid().transform(asTaskId),
  stepNumber: z.number().int().positive(),
  stepTitle: z.string().nullable(),
  bodyMarkdown: z.string(),
  createdAt: z.string(),
  updatedAt: z.string()
})

export const TaskDeviationListSchema = z.array(TaskDeviationSchema)

/** A deviation a session wrote (`deviation` SSE event); the console reads that task's deviations again. */
export const DeviationStreamEventSchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  stepNumber: z.number().int().positive()
})
