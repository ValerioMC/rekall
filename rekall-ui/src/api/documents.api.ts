import { apiClient, request } from './client'
import {
  DocumentListSchema,
  DocumentMatchListSchema,
  DocumentSchema,
  ExportResultSchema,
  ImportReportSchema
} from './schemas/record.schema'
import type { DocumentMatch, RekallDocument } from '@/model/records'
import type { DocumentId, EntityName, RecordId } from '@/model/branded'

export async function fetchDocuments(entity: EntityName, recordId: RecordId): Promise<RekallDocument[]> {
  return request(async () =>
    DocumentListSchema.parse(await apiClient('/api/documents', { query: { entity, recordId } }))
  )
}

export async function createDocument(input: {
  entityName: EntityName
  recordId: RecordId
  title: string
  kind: string
  bodyMarkdown: string
}): Promise<RekallDocument> {
  return request(async () =>
    DocumentSchema.parse(await apiClient('/api/documents', { method: 'POST', body: input }))
  )
}

export async function updateDocument(
  id: DocumentId,
  input: { title: string; kind: string; bodyMarkdown: string }
): Promise<RekallDocument> {
  return request(async () =>
    DocumentSchema.parse(await apiClient(`/api/documents/${id}`, { method: 'PUT', body: input }))
  )
}

export async function deleteDocument(id: DocumentId): Promise<void> {
  await request(() => apiClient(`/api/documents/${id}`, { method: 'DELETE' }))
}

export async function searchDocuments(query: string): Promise<DocumentMatch[]> {
  return request(async () =>
    DocumentMatchListSchema.parse(await apiClient('/api/documents/search', { query: { query } }))
  )
}

export async function importFolder(path: string) {
  return request(async () =>
    ImportReportSchema.parse(await apiClient('/api/maintenance/import', { method: 'POST', body: { path } }))
  )
}

export async function exportFolder(path: string) {
  return request(async () =>
    ExportResultSchema.parse(await apiClient('/api/maintenance/export', { method: 'POST', body: { path } }))
  )
}
