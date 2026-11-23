import { z } from 'zod'
import { asDecisionId, asTaskId } from '@/model/branded'

const DecisionOptionSchema = z.object({
  label: z.string(),
  effect: z.string().nullable(),
  recommended: z.boolean()
})

const DecisionLineSchema = z.object({
  speaker: z.enum(['CLAUDE', 'PERSON']),
  text: z.string(),
  spokenAt: z.string()
})

const DecisionDetailSchema = z.object({
  text: z.string(),
  addedAt: z.string()
})

/** A decision as the routes answer it; the task it belongs to is the one the request named. */
export const DecisionResponseSchema = z.object({
  id: z.string().uuid().transform(asDecisionId),
  key: z.string(),
  topic: z.string().nullable(),
  question: z.string(),
  why: z.string().nullable(),
  stepNumber: z.number().int().positive().nullable(),
  stepTitle: z.string().nullable(),
  state: z.enum(['OPEN', 'ANSWERED', 'ON_HOLD']),
  answer: z.string().nullable(),
  holdNote: z.string().nullable(),
  previousAnswer: z.string().nullable(),
  changedAt: z.string().nullable(),
  source: z.enum(['TERMINAL', 'CHAT', 'PERSON']),
  options: z.array(DecisionOptionSchema),
  lines: z.array(DecisionLineSchema),
  details: z.array(DecisionDetailSchema)
})

export const TaskDecisionsResponseSchema = z.object({
  decisions: z.array(DecisionResponseSchema),
  sessionBlock: z.string()
})

/** A change to a task's decisions (`decision` SSE frame); the console reads that task's decisions again. */
export const DecisionStreamEventSchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  key: z.string()
})
