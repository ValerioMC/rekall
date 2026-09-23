import { z } from 'zod'
import { asTaskId } from '@/model/branded'
import { RUN_QUEUE_ITEM_STATES, RUN_QUEUE_STATES, asRunQueueItemId } from '@/model/runQueue'

export const RunQueueItemSchema = z.object({
  id: z.string().uuid().transform(asRunQueueItemId),
  taskId: z.string().uuid().transform(asTaskId),
  taskTitle: z.string(),
  taskLabel: z.string(),
  projectLabel: z.string(),
  anchor: z.string(),
  position: z.number().int(),
  state: z.enum(RUN_QUEUE_ITEM_STATES),
  detail: z.string().nullable(),
  startedAt: z.string().nullable(),
  finishedAt: z.string().nullable()
})

export const RunQueueSchema = z.object({
  state: z.enum(RUN_QUEUE_STATES),
  startAt: z.string().nullable(),
  ceilingPercent: z.number().int().nullable(),
  skipPermissions: z.boolean(),
  model: z.string().nullable(),
  effort: z.string().nullable(),
  holdUntil: z.string().nullable(),
  holdReason: z.string().nullable(),
  items: z.array(RunQueueItemSchema),
  updatedAt: z.string()
})
