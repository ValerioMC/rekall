import { z } from 'zod'
import { asTaskId, asTaskStepId, asTerminalId } from '@/model/branded'

export const TerminalSchema = z.object({
  id: z.string().uuid().transform(asTerminalId),
  taskId: z.string().uuid().transform(asTaskId),
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  anchors: z.string(),
  workingDir: z.string(),
  projectLabel: z.string(),
  taskLabel: z.string(),
  taskTitle: z.string(),
  skipPermissions: z.boolean(),
  model: z.string().nullable(),
  effort: z.string().nullable(),
  live: z.boolean(),
  startedAt: z.string(),
  lastActivityAt: z.string()
})

export const TerminalListSchema = z.array(TerminalSchema)
