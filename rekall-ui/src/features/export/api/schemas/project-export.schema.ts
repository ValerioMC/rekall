import { z } from 'zod'
import { asCompanyId, asProjectExportId, asProjectId } from '@/model/branded'
import { EXPORT_FORMATS } from '@/features/export/model/project-export'

export const ProjectExportSchema = z.object({
  id: z.string().uuid().transform(asProjectExportId),
  projectId: z.string().uuid().transform(asProjectId),
  projectLabel: z.string(),
  projectTitle: z.string(),
  companyId: z.string().uuid().transform(asCompanyId),
  companyName: z.string(),
  format: z.enum(EXPORT_FORMATS),
  status: z.enum(['DONE', 'FAILED']),
  summary: z.string(),
  fileName: z.string(),
  sizeBytes: z.number().int(),
  diagramsMissing: z.number().int(),
  failureKind: z.enum(['RENDER', 'STORAGE']).nullish().transform((value) => value ?? null),
  failureMessage: z.string().nullish().transform((value) => value ?? null),
  createdAt: z.string()
})

export const ProjectExportListSchema = z.array(ProjectExportSchema)
