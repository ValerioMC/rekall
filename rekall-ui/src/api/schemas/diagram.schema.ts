import { z } from 'zod'
import { asDiagramId, asProjectId, asTaskId } from '@/model/branded'
import type { GraphEdge, GraphNode, SemanticGraph, SourceLocation } from '@/model/diagram'

const provenance = z.enum(['observed', 'inferred', 'documented', 'stated'])
const metadata = z.record(z.unknown())

const SourceLocationSchema: z.ZodType<SourceLocation, z.ZodTypeDef, unknown> = z.object({
  file: z.string(),
  startLine: z.number().int().nullish().transform((value) => value ?? null),
  endLine: z.number().int().nullish().transform((value) => value ?? null),
  symbol: z.string().nullish().transform((value) => value ?? null)
})

// The server omits empty optional fields to keep stored graphs small; they are filled in here
// so every consumer reads one complete shape.
const GraphNodeSchema: z.ZodType<GraphNode, z.ZodTypeDef, unknown> = z.object({
  id: z.string(),
  kind: z.string(),
  title: z.string(),
  description: z.string().nullish().transform((value) => value ?? null),
  sources: z.array(SourceLocationSchema).default([]),
  provenance: provenance.nullish().transform((value) => value ?? null),
  confidence: z.number().nullish().transform((value) => value ?? null),
  metadata: metadata.default({})
})

const GraphEdgeSchema: z.ZodType<GraphEdge, z.ZodTypeDef, unknown> = z.object({
  id: z.string(),
  from: z.string(),
  to: z.string(),
  relation: z.string(),
  label: z.string().nullish().transform((value) => value ?? null),
  provenance: provenance.nullish().transform((value) => value ?? null),
  confidence: z.number().nullish().transform((value) => value ?? null),
  metadata: metadata.default({})
})

export const SemanticGraphSchema: z.ZodType<SemanticGraph, z.ZodTypeDef, unknown> = z.object({
  format: z.string(),
  version: z.number().int(),
  nodes: z.array(GraphNodeSchema),
  edges: z.array(GraphEdgeSchema).default([])
})

const summaryShape = {
  id: z.string().uuid().transform(asDiagramId),
  projectId: z.string().uuid().transform(asProjectId),
  taskId: z.string().uuid().transform(asTaskId).nullable(),
  title: z.string(),
  question: z.string(),
  nodeCount: z.number().int(),
  edgeCount: z.number().int(),
  createdAt: z.string(),
  updatedAt: z.string()
}

export const DiagramSummarySchema = z.object(summaryShape)

export const DiagramSummaryListSchema = z.array(DiagramSummarySchema)

export const DiagramSchema = z.object({ ...summaryShape, graph: SemanticGraphSchema })

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
