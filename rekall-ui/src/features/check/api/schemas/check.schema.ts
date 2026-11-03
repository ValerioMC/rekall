import { z } from 'zod'
import { asCheckResultId, asDoneCheckId, asProjectId, asTaskId, asTaskStepId } from '@/model/branded'

export const DoneCheckSchema = z.object({
  id: z.string().uuid().transform(asDoneCheckId),
  projectId: z.string().uuid().transform(asProjectId),
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  label: z.string(),
  command: z.string(),
  folder: z.string().nullable(),
  timeoutSeconds: z.number().int(),
  position: z.number().int(),
  createdAt: z.string(),
  updatedAt: z.string()
})

export const DoneCheckListSchema = z.array(DoneCheckSchema)

export const CheckResultSchema = z.object({
  id: z.string().uuid().transform(asCheckResultId),
  checkId: z.string().uuid().transform(asDoneCheckId),
  taskId: z.string().uuid().transform(asTaskId),
  stepId: z.string().uuid().transform(asTaskStepId).nullable(),
  label: z.string(),
  command: z.string(),
  passed: z.boolean(),
  timedOut: z.boolean(),
  exitCode: z.number().int().nullable(),
  outputTail: z.string(),
  ranAt: z.string(),
  durationMs: z.number().int()
})

export const CheckResultListSchema = z.array(CheckResultSchema)

export const ProjectCheckStreamEventSchema = z.object({
  projectId: z.string().uuid().transform(asProjectId)
})

export const TaskCheckResultStreamEventSchema = z.object({
  taskId: z.string().uuid().transform(asTaskId)
})
