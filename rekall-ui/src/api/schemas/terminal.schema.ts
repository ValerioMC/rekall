import { z } from 'zod'
import { asProjectId, asTaskId, asTaskStepId, asTerminalId } from '@/model/branded'

export const TerminalSchema = z.object({
  id: z.string().uuid().transform(asTerminalId),
  projectId: z.string().uuid().transform(asProjectId),
  taskId: z.string().uuid().transform(asTaskId).nullable(),
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  anchors: z.string(),
  workingDir: z.string(),
  projectLabel: z.string(),
  taskLabel: z.string().nullable(),
  taskTitle: z.string().nullable(),
  skipPermissions: z.boolean(),
  model: z.string().nullable(),
  effort: z.string().nullable(),
  live: z.boolean(),
  startedAt: z.string(),
  lastActivityAt: z.string()
})

export const TerminalListSchema = z.array(TerminalSchema)
