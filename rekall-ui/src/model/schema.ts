import type { EntityId, EntityName, FieldId, RelationId } from './branded'

export type FieldType =
  | 'TEXT'
  | 'LONG_TEXT'
  | 'MARKDOWN'
  | 'INTEGER'
  | 'DECIMAL'
  | 'BOOLEAN'
  | 'DATE'
  | 'TIMESTAMP'
  | 'ENUM'
  | 'TAGS'
  | 'REFERENCE'

export type EntityStatus = 'DRAFT' | 'APPLIED' | 'MODIFIED'
export type RelationKind = 'MANY_TO_ONE' | 'MANY_TO_MANY'
export type OnDeleteAction = 'RESTRICT' | 'CASCADE' | 'SET_NULL'

export type EntityField = Readonly<{
  id: FieldId
  columnName: string
  label: string
  description: string
  type: FieldType
  nullable: boolean
  defaultValue: string | null
  length: number | null
  precision: number | null
  scale: number | null
  enumValues: readonly string[]
  position: number
}>

export type Entity = Readonly<{
  id: EntityId
  physicalName: EntityName
  label: string
  labelPlural: string
  description: string
  aliases: readonly string[]
  displayFieldId: FieldId | null
  status: EntityStatus
  fields: readonly EntityField[]
}>

export type Relation = Readonly<{
  id: RelationId
  sourceTableId: EntityId
  sourceTableName: EntityName
  targetTableId: EntityId
  targetTableName: EntityName
  kind: RelationKind
  sourceFieldId: FieldId | null
  joinTableName: string | null
  onDelete: OnDeleteAction
  description: string
}>

export const FIELD_TYPES: readonly FieldType[] = [
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
] as const
