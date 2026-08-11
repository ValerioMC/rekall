import type { DocumentId, EntityName, RecordId } from './branded'

/**
 * One row of a generated table.
 *
 * `values` is deliberately loose: the shape is defined at runtime by the meta-model, so the
 * compiler has nothing to check it against. A resolved reference holds a nested record rather
 * than a bare uuid, which is what makes an answer readable.
 */
export type EntityRecord = Readonly<{
  id: RecordId
  entityName: EntityName
  label: string
  values: Readonly<Record<string, RecordValue>>
  createdAt: Date
  updatedAt: Date
}>

export type RecordValue = string | number | boolean | readonly string[] | EntityRecord | null

export type RecordPage = Readonly<{
  records: readonly EntityRecord[]
  total: number
  limit: number
  offset: number
}>

export type RekallDocument = Readonly<{
  id: DocumentId
  entityName: EntityName
  recordId: RecordId
  title: string
  kind: string
  bodyMarkdown: string
  sourcePath: string | null
  position: number
  updatedAt: Date
}>

export type DocumentMatch = Readonly<{
  documentId: DocumentId
  entityName: EntityName
  recordId: RecordId
  title: string
  kind: string
  /** Server-generated `ts_headline` fragment; contains `<b>` around the hits and nothing else. */
  excerpt: string
  rank: number
}>

export const DOCUMENT_KINDS = ['context', 'notes', 'architecture', 'report', 'other'] as const

export function isResolvedReference(value: RecordValue): value is EntityRecord {
  return typeof value === 'object' && value !== null && !Array.isArray(value) && 'label' in value
}
