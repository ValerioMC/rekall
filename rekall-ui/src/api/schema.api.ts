import { apiClient, request } from './client'
import {
  EntityFieldSchema,
  EntityListSchema,
  EntitySchema,
  RelationListSchema,
  RelationSchema
} from './schemas/schema.schema'
import type { Entity, EntityField, FieldType, OnDeleteAction, Relation, RelationKind } from '@/model/schema'
import type { EntityId, FieldId, RelationId } from '@/model/branded'

export type CreateEntityInput = Readonly<{
  physicalName: string
  label: string
  labelPlural: string
  description: string
  aliases: readonly string[]
}>

export type UpdateEntityInput = Readonly<{
  label: string
  labelPlural: string
  description: string
  aliases: readonly string[]
  displayFieldId: FieldId | null
}>

export type FieldInput = Readonly<{
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
}>

export type CreateRelationInput = Readonly<{
  sourceTableId: EntityId
  targetTableId: EntityId
  kind: RelationKind
  sourceFieldId: FieldId | null
  joinTableName: string | null
  onDelete: OnDeleteAction
  description: string
}>

export async function fetchEntities(): Promise<Entity[]> {
  return request(async () => EntityListSchema.parse(await apiClient('/api/meta/tables')))
}

export async function fetchEntity(id: EntityId): Promise<Entity> {
  return request(async () => EntitySchema.parse(await apiClient(`/api/meta/tables/${id}`)))
}

export async function createEntity(input: CreateEntityInput): Promise<Entity> {
  return request(async () =>
    EntitySchema.parse(await apiClient('/api/meta/tables', { method: 'POST', body: input }))
  )
}

export async function updateEntity(id: EntityId, input: UpdateEntityInput): Promise<Entity> {
  return request(async () =>
    EntitySchema.parse(await apiClient(`/api/meta/tables/${id}`, { method: 'PUT', body: input }))
  )
}

export async function deleteEntity(id: EntityId): Promise<void> {
  await request(() => apiClient(`/api/meta/tables/${id}`, { method: 'DELETE' }))
}

export async function addField(entityId: EntityId, input: FieldInput): Promise<EntityField> {
  return request(async () =>
    EntityFieldSchema.parse(
      await apiClient(`/api/meta/tables/${entityId}/fields`, { method: 'POST', body: input })
    )
  )
}

export async function updateField(
  fieldId: FieldId,
  input: Omit<FieldInput, 'columnName'>
): Promise<EntityField> {
  return request(async () =>
    EntityFieldSchema.parse(await apiClient(`/api/meta/fields/${fieldId}`, { method: 'PUT', body: input }))
  )
}

export async function deleteField(fieldId: FieldId): Promise<void> {
  await request(() => apiClient(`/api/meta/fields/${fieldId}`, { method: 'DELETE' }))
}

export async function fetchRelations(): Promise<Relation[]> {
  return request(async () => RelationListSchema.parse(await apiClient('/api/meta/relations')))
}

export async function createRelation(input: CreateRelationInput): Promise<Relation> {
  return request(async () =>
    RelationSchema.parse(await apiClient('/api/meta/relations', { method: 'POST', body: input }))
  )
}

export async function deleteRelation(id: RelationId): Promise<void> {
  await request(() => apiClient(`/api/meta/relations/${id}`, { method: 'DELETE' }))
}
