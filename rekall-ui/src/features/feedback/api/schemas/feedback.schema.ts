import { z } from 'zod'
import { asFeedbackId, asTaskId } from '@/model/branded'

export const FeedbackSchema = z.object({
  id: z.string().uuid().transform(asFeedbackId),
  taskId: z.string().uuid().transform(asTaskId),
  seq: z.number().int().positive(),
  kind: z.enum(['IMPLEMENTATION', 'PLAN']),
  state: z.enum(['PENDING', 'RUNNING', 'APPLIED']),
  quotedMarkdown: z.string().nullable(),
  bodyMarkdown: z.string(),
  replyMarkdown: z.string().nullable(),
  createdAt: z.string(),
  appliedAt: z.string().nullable(),
  pass: z.number().int().positive().nullable(),
  takenAt: z.string().nullable()
})

export const FeedbackListSchema = z.array(FeedbackSchema)

/** A change to a task's thread (`feedback` SSE frame); the console reads that task's thread again. */
export const FeedbackStreamEventSchema = z.object({
  taskId: z.string().uuid().transform(asTaskId)
})
