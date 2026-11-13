import { z } from 'zod'
import { asTaskId } from '@/model/branded'
import {
  RUN_QUEUE_ITEM_STATES,
  RUN_QUEUE_STATES,
  RUN_QUEUE_STEP_STATES,
  asRunQueueItemId
} from '@/features/queue/model/runQueue'

export const RunQueueStepSchema = z.object({
  title: z.string(),
  state: z.enum(RUN_QUEUE_STEP_STATES),
  current: z.boolean()
})

export const RunQueueWaitSchema = z.object({
  anchor: z.string(),
  title: z.string(),
  awaitingReview: z.boolean(),
  inQueue: z.boolean()
})

export const RunQueueItemSchema = z.object({
  id: z.string().uuid().transform(asRunQueueItemId),
  taskId: z.string().uuid().transform(asTaskId),
  taskTitle: z.string(),
  taskCode: z.string(),
  projectLabel: z.string(),
  anchor: z.string(),
  position: z.number().int(),
  state: z.enum(RUN_QUEUE_ITEM_STATES),
  detail: z.string().nullable(),
  startedAt: z.string().nullable(),
  finishedAt: z.string().nullable(),
  steps: z.array(RunQueueStepSchema).default([]),
  waitingFor: z.array(RunQueueWaitSchema).default([])
})

export const RunQueueSchema = z.object({
  state: z.enum(RUN_QUEUE_STATES),
  startAt: z.string().nullable(),
  ceilingPercent: z.number().int().nullable(),
  skipPermissions: z.boolean(),
  model: z.string().nullable(),
  effort: z.string().nullable(),
  sessionKind: z.enum(['TERMINAL', 'CHAT']),
  parallelSessions: z.number().int().min(1).default(1),
  holdUntil: z.string().nullable(),
  holdReason: z.string().nullable(),
  items: z.array(RunQueueItemSchema),
  updatedAt: z.string()
})
