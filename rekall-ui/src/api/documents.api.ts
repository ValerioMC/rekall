import { apiClient, request } from './client'
import { DocumentListSchema, DocumentSchema } from './schemas/catalog.schema'
import type { RekallDocument } from '@/model/catalog'
import type { DocumentId, TaskId } from '@/model/branded'

export interface DocumentInput {
  title: string
  kind: string
  bodyMarkdown: string
  taskIds: readonly TaskId[]
}

export async function fetchDocuments(taskId: TaskId): Promise<RekallDocument[]> {
  return request(async () =>
    DocumentListSchema.parse(await apiClient('/api/documents', { query: { taskId } }))
  )
}

export async function fetchAllDocuments(): Promise<RekallDocument[]> {
  return request(async () => DocumentListSchema.parse(await apiClient('/api/documents')))
}

export async function createDocument(input: DocumentInput): Promise<RekallDocument> {
  return request(async () =>
    DocumentSchema.parse(await apiClient('/api/documents', { method: 'POST', body: input }))
  )
}

export async function updateDocument(
  id: DocumentId,
  input: DocumentInput
): Promise<RekallDocument> {
  return request(async () =>
    DocumentSchema.parse(await apiClient(`/api/documents/${id}`, { method: 'PUT', body: input }))
  )
}

export async function deleteDocument(id: DocumentId): Promise<void> {
  await request(() => apiClient(`/api/documents/${id}`, { method: 'DELETE' }))
}

export async function searchDocuments(query: string): Promise<RekallDocument[]> {
  return request(async () =>
    DocumentListSchema.parse(await apiClient('/api/documents/search', { query: { query } }))
  )
}
