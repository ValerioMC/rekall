import { z } from 'zod'
import { asTaskId, asTaskStepId } from '@/model/branded'

export const CommitReferenceSchema = z.object({
  id: z.string().uuid(),
  taskId: z.string().uuid().transform(asTaskId),
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  stepTitle: z.string().nullable(),
  commitHash: z.string(),
  comment: z.string(),
  inContext: z.boolean(),
  createdAt: z.string()
})

export const CommitReferenceStreamEventSchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  reference: CommitReferenceSchema
})

export const CommitReferenceDiffSchema = z.object({
  diff: z.string().nullable()
})

export const RecentCommitSchema = z.object({
  hash: z.string(),
  subject: z.string(),
  committedAt: z.string()
})
