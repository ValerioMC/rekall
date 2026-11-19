import { apiClient, request } from '../../../api/client'
import { DocumentListSchema, DocumentSchema } from '../../catalog/api/schemas/catalog.schema'
import type { DocumentContextMode, RekallDocument } from '@/features/notes/model/document'
import type { DocumentId, TaskId } from '@/model/branded'

export interface DocumentInput {
  title: string
  kind: string
  bodyMarkdown: string
  /** The task the note belongs to; an update has to name the one it already has. */
  taskId: TaskId
  /** Left out, a new note is sent in full and an existing one keeps its mode. */
  contextMode?: DocumentContextMode
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
