import { z } from 'zod'
import { asTaskId, asTaskStepId } from '@/model/branded'

export const CommitReferenceSchema = z.object({
  id: z.string().uuid(),
  taskId: z.string().uuid().transform(asTaskId),
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  stepTitle: z.string().nullable(),
  commitHash: z.string(),
  comment: z.string(),
  createdAt: z.string()
})
