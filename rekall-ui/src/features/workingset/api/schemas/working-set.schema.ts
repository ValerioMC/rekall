import { z } from 'zod'
import { asProjectId, asTaskId } from '@/model/branded'

export const OpenPlaceSchema = z.object({
  projectId: z.string().uuid().transform(asProjectId),
  taskId: z.string().uuid().transform(asTaskId).nullable()
})

const OpenTaskSchema = z.object({
  taskId: z.string().uuid().transform(asTaskId),
  projectId: z.string().uuid().transform(asProjectId)
})

export const WorkingSetSchema = z.object({
  tasks: z.array(OpenTaskSchema),
  places: z.array(OpenPlaceSchema)
})
