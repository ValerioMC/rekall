import { z } from 'zod'
import { asDiagramId, asProjectId, asTaskId } from '@/model/branded'
import { DIAGRAM_KINDS, type DiagramSheet, type SourceLocation } from '@/features/diagram/model/diagram'

const kind = z.enum(DIAGRAM_KINDS)

// The server omits empty optional fields to keep stored documents small; they are filled in
// here so every consumer reads one complete shape.
const SourceLocationSchema: z.ZodType<SourceLocation, z.ZodTypeDef, unknown> = z.object({
  file: z.string(),
  startLine: z.number().int().nullish().transform((value) => value ?? null),
  endLine: z.number().int().nullish().transform((value) => value ?? null),
  symbol: z.string().nullish().transform((value) => value ?? null)
})

const DiagramSheetSchema: z.ZodType<DiagramSheet, z.ZodTypeDef, unknown> = z.object({
  kind,
  title: z.string().nullish().transform((value) => value ?? null),
  mermaid: z.string(),
  links: z.record(z.array(SourceLocationSchema)).default({})
})

const summaryShape = {
  id: z.string().uuid().transform(asDiagramId),
  projectId: z.string().uuid().transform(asProjectId),
  taskId: z.string().uuid().transform(asTaskId).nullable(),
  title: z.string(),
  question: z.string(),
  kinds: z.array(kind),
  createdAt: z.string(),
  updatedAt: z.string()
}

export const DiagramSummarySchema = z.object(summaryShape)

export const DiagramSummaryListSchema = z.array(DiagramSummarySchema)

export const DiagramSchema = z.object({ ...summaryShape, sheets: z.array(DiagramSheetSchema) })

export const SourceExcerptSchema = z.object({
  file: z.string(),
  language: z.string().nullable(),
  highlightStart: z.number().int().nullable(),
  highlightEnd: z.number().int().nullable(),
  totalLines: z.number().int(),
  lines: z.array(z.object({ number: z.number().int(), text: z.string() })),
  truncated: z.boolean()
})

export const DiagramStreamEventSchema = z.object({
  diagramId: z.string().uuid().transform(asDiagramId),
  diagram: DiagramSummarySchema.nullable(),
  deleted: z.boolean()
})
