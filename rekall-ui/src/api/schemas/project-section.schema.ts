import { z } from 'zod'
import { asProjectId, asProjectSectionId } from '@/model/branded'

export const ProjectSectionSchema = z.object({
  id: z.string().uuid().transform(asProjectSectionId),
  projectId: z.string().uuid().transform(asProjectId),
  title: z.string(),
  bodyMarkdown: z.string(),
  position: z.number().int(),
  createdAt: z.string(),
  updatedAt: z.string()
})

export const ProjectSectionListSchema = z.array(ProjectSectionSchema)

export const ProjectSectionStreamEventSchema = z.object({
  projectId: z.string().uuid().transform(asProjectId)
})
