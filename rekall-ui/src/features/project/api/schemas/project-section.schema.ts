import { z } from 'zod'
import { asProjectId, asProjectSectionId, asSectionContextId } from '@/model/branded'

export const ProjectSectionSchema = z.object({
  id: z.string().uuid().transform(asProjectSectionId),
  projectId: z.string().uuid().transform(asProjectId),
  contextId: z.string().uuid().transform(asSectionContextId),
  title: z.string(),
  bodyMarkdown: z.string(),
  position: z.number().int(),
  createdAt: z.string(),
  updatedAt: z.string()
})

export const ProjectSectionListSchema = z.array(ProjectSectionSchema)

export const SectionContextSchema = z.object({
  id: z.string().uuid().transform(asSectionContextId),
  projectId: z.string().uuid().transform(asProjectId),
  title: z.string(),
  position: z.number().int(),
  createdAt: z.string(),
  updatedAt: z.string()
})

export const SectionContextListSchema = z.array(SectionContextSchema)

export const SectionLayoutSchema = z.object({
  contexts: SectionContextListSchema,
  sections: ProjectSectionListSchema
})

export const ProjectSectionStreamEventSchema = z.object({
  projectId: z.string().uuid().transform(asProjectId)
})
