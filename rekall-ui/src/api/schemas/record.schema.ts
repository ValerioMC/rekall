import { z } from 'zod'
import { asDocumentId, asEntityName, asRecordId } from '@/model/branded'
import type { EntityRecord, RecordValue } from '@/model/records'

const recordId = z.string().uuid().transform(asRecordId)
const documentId = z.string().uuid().transform(asDocumentId)
const entityName = z.string().min(1).transform(asEntityName)

/**
 * A record value is whatever the meta-model says it is, plus a nested record when a reference
 * has been resolved. The recursion is why this needs an explicit type annotation: Zod cannot
 * infer a self-referential schema on its own.
 */
const RecordValueSchema: z.ZodType<RecordValue, z.ZodTypeDef, unknown> = z.lazy(() =>
  z.union([z.string(), z.number(), z.boolean(), z.array(z.string()), EntityRecordSchema, z.null()])
)

export const EntityRecordSchema: z.ZodType<EntityRecord, z.ZodTypeDef, unknown> = z.lazy(() =>
  z.object({
    id: recordId,
    entityName,
    label: z.string(),
    values: z.record(z.string(), RecordValueSchema),
    createdAt: z.coerce.date(),
    updatedAt: z.coerce.date()
  })
)

export const RecordPageSchema = z.object({
  records: z.array(EntityRecordSchema),
  total: z.number().int(),
  limit: z.number().int(),
  offset: z.number().int()
})

export const DocumentSchema = z.object({
  id: documentId,
  entityName,
  recordId,
  title: z.string(),
  kind: z.string(),
  bodyMarkdown: z.string(),
  sourcePath: z.string().nullable(),
  position: z.number().int(),
  updatedAt: z.coerce.date()
})

export const DocumentListSchema = z.array(DocumentSchema)

export const DocumentMatchSchema = z.object({
  documentId,
  entityName,
  recordId,
  title: z.string(),
  kind: z.string(),
  excerpt: z.string(),
  rank: z.number()
})

export const DocumentMatchListSchema = z.array(DocumentMatchSchema)

export const ImportReportSchema = z.object({
  projectsCreated: z.number().int(),
  tasksCreated: z.number().int(),
  documentsCreated: z.number().int(),
  warnings: z.array(z.string())
})

export const ExportResultSchema = z.object({ filesWritten: z.number().int() })
