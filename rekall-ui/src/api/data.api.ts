import { apiClient, request } from './client'
import { EntityRecordSchema, RecordPageSchema } from './schemas/record.schema'
import type { EntityRecord, RecordPage, RecordValue } from '@/model/records'
import type { EntityName, RecordId } from '@/model/branded'

export type RecordInput = Readonly<Record<string, RecordValue>>

export async function fetchRecords(
  entity: EntityName,
  search = '',
  limit = 50,
  offset = 0
): Promise<RecordPage> {
  return request(async () =>
    RecordPageSchema.parse(await apiClient(`/api/data/${entity}`, { query: { search, limit, offset } }))
  )
}

export async function fetchRecord(entity: EntityName, id: RecordId): Promise<EntityRecord> {
  return request(async () => EntityRecordSchema.parse(await apiClient(`/api/data/${entity}/${id}`)))
}

export async function createRecord(entity: EntityName, values: RecordInput): Promise<EntityRecord> {
  return request(async () =>
    EntityRecordSchema.parse(await apiClient(`/api/data/${entity}`, { method: 'POST', body: values }))
  )
}

export async function updateRecord(
  entity: EntityName,
  id: RecordId,
  values: RecordInput
): Promise<EntityRecord> {
  return request(async () =>
    EntityRecordSchema.parse(await apiClient(`/api/data/${entity}/${id}`, { method: 'PUT', body: values }))
  )
}

export async function deleteRecord(entity: EntityName, id: RecordId): Promise<void> {
  await request(() => apiClient(`/api/data/${entity}/${id}`, { method: 'DELETE' }))
}
