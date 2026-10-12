import { z } from 'zod'
import { RESOURCE_KINDS, type MemoryGroup, type ResourceCatalog, type ResourceFile, type ResourceSummary } from '@/model/claude-resources'

const SummarySchema: z.ZodType<ResourceSummary, z.ZodTypeDef, unknown> = z.object({
  path: z.string(),
  kind: z.enum(RESOURCE_KINDS),
  name: z.string(),
  description: z.string().nullable(),
  memoryType: z.string().nullable(),
  bytes: z.number().int(),
  modifiedAt: z.string(),
  editable: z.boolean()
})

const MemoryGroupSchema: z.ZodType<MemoryGroup, z.ZodTypeDef, unknown> = z.object({
  id: z.string(),
  global: z.boolean(),
  workingDirectory: z.string().nullable(),
  resolved: z.boolean(),
  folderExists: z.boolean(),
  index: SummarySchema.nullable(),
  memories: z.array(SummarySchema)
})

export const ResourceCatalogSchema: z.ZodType<ResourceCatalog, z.ZodTypeDef, unknown> = z.object({
  root: z.string(),
  instructions: SummarySchema.nullable(),
  skills: z.array(SummarySchema),
  commands: z.array(SummarySchema),
  agents: z.array(SummarySchema),
  memoryGroups: z.array(MemoryGroupSchema)
})

export const ResourceFileSchema: z.ZodType<ResourceFile, z.ZodTypeDef, unknown> = z.object({
  summary: SummarySchema,
  body: z.string()
})
