import { z } from 'zod'
import { asProjectComponentId, asProjectId } from '@/model/branded'

export const ProjectComponentSchema = z.object({
  id: z.string().uuid().transform(asProjectComponentId),
  projectId: z.string().uuid().transform(asProjectId),
  name: z.string(),
  folder: z.string(),
  description: z.string(),
  position: z.number().int(),
  createdAt: z.string(),
  updatedAt: z.string()
})

export const ProjectComponentListSchema = z.array(ProjectComponentSchema)
