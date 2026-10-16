import { z } from 'zod'
import {
  RESOURCE_KINDS,
  type MemoryGroup,
  type ResourceCatalog,
  type ResourceFile,
  type ResourceSummary,
  type SkillPlacement,
  type SkillSource
} from '@/features/claude-resources/model/claude-resources'
import type { MemoryAnalysis, MemoryRef, ObsoleteMemory, PromotionCandidate } from '@/features/claude-resources/model/memory-analysis'

const SkillSourceSchema: z.ZodType<SkillSource, z.ZodTypeDef, unknown> = z.object({
  kind: z.enum(['own', 'synced', 'plugin']),
  name: z.string(),
  detail: z.string().nullable()
})


const SummarySchema: z.ZodType<ResourceSummary, z.ZodTypeDef, unknown> = z.object({
  path: z.string(),
  kind: z.enum(RESOURCE_KINDS),
  name: z.string(),
  description: z.string().nullable(),
  memoryType: z.string().nullable(),
  bytes: z.number().int(),
  modifiedAt: z.string(),
  editable: z.boolean(),
  source: SkillSourceSchema.nullable()
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

export const SkillPlacementsSchema: z.ZodType<readonly SkillPlacement[], z.ZodTypeDef, unknown> = z.array(
  z.object({ skillPath: z.string(), group: z.string() })
)

const MemoryRefSchema: z.ZodType<MemoryRef, z.ZodTypeDef, unknown> = z.object({
  path: z.string(),
  name: z.string(),
  groupId: z.string(),
  memoryType: z.string().nullable()
})

const PromotionSchema: z.ZodType<PromotionCandidate, z.ZodTypeDef, unknown> = z.object({
  topic: z.array(z.string()),
  memories: z.array(MemoryRefSchema),
  groupCount: z.number().int(),
  target: z.enum(['instructions', 'skill']),
  reason: z.string()
})

const ObsoleteSchema: z.ZodType<ObsoleteMemory, z.ZodTypeDef, unknown> = z.object({
  memory: MemoryRefSchema,
  coveredBy: z.object({
    kind: z.enum(['globalInstructions', 'projectInstructions', 'skill']),
    label: z.string(),
    location: z.string(),
    resourcePath: z.string().nullable()
  }),
  overlapPercent: z.number().int(),
  sharedPhrases: z.array(z.string())
})

export const MemoryAnalysisSchema: z.ZodType<MemoryAnalysis, z.ZodTypeDef, unknown> = z.object({
  memoriesRead: z.number().int(),
  documentsRead: z.number().int(),
  promotions: z.array(PromotionSchema),
  obsolete: z.array(ObsoleteSchema)
})
