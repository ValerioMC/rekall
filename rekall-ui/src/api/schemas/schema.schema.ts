import { z } from 'zod'
import { asEntityId, asEntityName, asFieldId, asRelationId } from '@/model/branded'

/**
 * Runtime validation of the schema endpoints.
 *
 * The brands are applied by `transform` rather than by a cast at the call site: this is the
 * one place that has actually checked the value is a uuid, so it is the only place entitled
 * to say what kind of id it is.
 */
const entityId = z.string().uuid().transform(asEntityId)
const fieldId = z.string().uuid().transform(asFieldId)
const relationId = z.string().uuid().transform(asRelationId)
const entityName = z.string().min(1).transform(asEntityName)

export const FieldTypeSchema = z.enum([
  'TEXT',
  'LONG_TEXT',
  'MARKDOWN',
  'INTEGER',
  'DECIMAL',
  'BOOLEAN',
  'DATE',
  'TIMESTAMP',
  'ENUM',
  'TAGS',
  'REFERENCE'
])

export const EntityFieldSchema = z.object({
  id: fieldId,
  columnName: z.string(),
  label: z.string(),
  description: z.string(),
  type: FieldTypeSchema,
  nullable: z.boolean(),
  defaultValue: z.string().nullable(),
  length: z.number().int().nullable(),
  precision: z.number().int().nullable(),
  scale: z.number().int().nullable(),
  enumValues: z.array(z.string()),
  position: z.number().int()
})

export const EntitySchema = z.object({
  id: entityId,
  physicalName: entityName,
  label: z.string(),
  labelPlural: z.string(),
  description: z.string(),
  aliases: z.array(z.string()),
  displayFieldId: fieldId.nullable(),
  status: z.enum(['DRAFT', 'APPLIED', 'MODIFIED']),
  fields: z.array(EntityFieldSchema)
})

export const RelationSchema = z.object({
  id: relationId,
  sourceTableId: entityId,
  sourceTableName: entityName,
  targetTableId: entityId,
  targetTableName: entityName,
  kind: z.enum(['MANY_TO_ONE', 'MANY_TO_MANY']),
  sourceFieldId: fieldId.nullable(),
  joinTableName: z.string().nullable(),
  onDelete: z.enum(['RESTRICT', 'CASCADE', 'SET_NULL']),
  description: z.string()
})

export const EntityListSchema = z.array(EntitySchema)
export const RelationListSchema = z.array(RelationSchema)
