import { z } from 'zod'
import { asDocumentId, asTaskId, asTaskStepId } from '@/model/branded'

export const SearchHitSchema = z.object({
  kind: z.enum(['DESCRIPTION', 'STEP', 'WRAPUP', 'NOTE']),
  taskId: z.string().uuid().transform(asTaskId).nullable(),
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  documentId: z.string().uuid().transform(asDocumentId).nullable(),
  title: z.string(),
  where: z.string(),
  excerpt: z.string()
})

export const SearchHitListSchema = z.array(SearchHitSchema)
