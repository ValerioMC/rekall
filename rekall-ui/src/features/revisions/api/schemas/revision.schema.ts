import { z } from 'zod'
import { asTaskId } from '@/model/branded'

const revisionKind = z.enum(['WRAPUP', 'DESCRIPTION'])

export const TaskRevisionSchema = z.object({
  id: z.string().uuid(),
  taskId: z.string().uuid().transform(asTaskId),
  kind: revisionKind,
  bodyMarkdown: z.string(),
  writtenBy: z.enum(['CLAUDE', 'HAND']).nullable(),
  writtenAt: z.string().nullable(),
  replacedAt: z.string()
})

export const TaskRevisionListSchema = z.array(TaskRevisionSchema)

export const RestoredRevisionSchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  kind: revisionKind,
  bodyMarkdown: z.string()
})
