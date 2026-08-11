import type { EntityField, FieldType } from './schema'
import type { EntityRecord, RecordValue } from './records'
import { isResolvedReference } from './records'

/** Human label for a field type. The raw enum reads badly in a form. */
const TYPE_LABELS: Readonly<Record<FieldType, string>> = {
  TEXT: 'Text',
  LONG_TEXT: 'Long text',
  MARKDOWN: 'Markdown',
  INTEGER: 'Number',
  DECIMAL: 'Decimal',
  BOOLEAN: 'Yes / no',
  DATE: 'Date',
  TIMESTAMP: 'Timestamp',
  ENUM: 'Choice',
  TAGS: 'Tags',
  REFERENCE: 'Reference'
}

export function typeLabel(type: FieldType): string {
  return TYPE_LABELS[type]
}

/** Renders any value as the single line a table cell can show. */
export function toCellText(value: RecordValue): string {
  if (value === null || value === undefined) return ''
  if (isResolvedReference(value)) return value.label
  if (Array.isArray(value)) return value.join(', ')
  return String(value)
}

/**
 * Flattens a record into the shape a form edits.
 *
 * Reads resolve references into whole records, but a form edits the id that points at them, so
 * the two representations have to be converted rather than shared.
 */
export function toFormModel(record: EntityRecord): Record<string, RecordValue> {
  return Object.fromEntries(
    Object.entries(record.values).map(([column, value]) => [
      column,
      isResolvedReference(value) ? value.id : value
    ])
  )
}

/** Suggests a physical name from what the user typed as the label. */
export function toPhysicalName(label: string): string {
  return label
    .toLowerCase()
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '')
    .slice(0, 63)
}

export function splitList(raw: string): string[] {
  return raw
    .split(',')
    .map((part) => part.trim())
    .filter((part) => part.length > 0)
}

/** Fields shown as columns in a listing. More than this and the table stops being readable. */
export function previewFields(fields: readonly EntityField[]): readonly EntityField[] {
  return fields.filter((field) => field.type !== 'MARKDOWN' && field.type !== 'LONG_TEXT').slice(0, 4)
}
