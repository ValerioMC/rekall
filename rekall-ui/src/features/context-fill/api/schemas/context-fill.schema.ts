import { z } from 'zod'
import { asTaskId, asTaskStepId } from '@/model/branded'

export const ContextFillSchema = z.object({
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  usedTokens: z.number().int().nonnegative(),
  windowTokens: z.number().int().nonnegative(),
  percent: z.number().int().min(0).max(100),
  recordedAt: z.string()
})

export const ContextFillListSchema = z.array(ContextFillSchema)

export const ContextFillStreamEventSchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  fills: ContextFillListSchema
})
