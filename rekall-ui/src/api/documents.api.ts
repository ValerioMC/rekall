import { apiClient, request } from './client'
import { DocumentListSchema, DocumentSchema } from './schemas/catalog.schema'
import type { RekallDocument } from '@/model/catalog'
import type { DocumentId, TaskId } from '@/model/branded'

export interface DocumentInput {
  title: string
  kind: string
  bodyMarkdown: string
  /**
   * Every task the note belongs to, and never empty: a note on no task cannot be reached from
   * anywhere, so the server refuses it rather than keeping a row nothing can show.
   */
  taskIds: readonly TaskId[]
}

export async function fetchDocuments(taskId: TaskId): Promise<RekallDocument[]> {
  return request(async () =>
    DocumentListSchema.parse(await apiClient('/api/documents', { query: { taskId } }))
  )
}

/** Every note, most recently written first. The console keeps the whole set in memory. */
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

/** Removes the note from every task at once. Detaching it from one is an update. */
export async function deleteDocument(id: DocumentId): Promise<void> {
  await request(() => apiClient(`/api/documents/${id}`, { method: 'DELETE' }))
}

export async function searchDocuments(query: string): Promise<RekallDocument[]> {
  return request(async () =>
    DocumentListSchema.parse(await apiClient('/api/documents/search', { query: { query } }))
  )
}
