import { apiClient, request } from './client'
import { DocumentListSchema, DocumentSchema } from './schemas/catalog.schema'
import type { RekallDocument } from '@/model/catalog'
import type { DocumentId, EnvironmentId, ProjectId, TaskId } from '@/model/branded'

/** Exactly one owner, which is what the server and the check constraint both require. */
export type DocumentOwner =
  | { projectId: ProjectId }
  | { taskId: TaskId }
  | { environmentId: EnvironmentId }

export async function fetchDocuments(owner: DocumentOwner): Promise<RekallDocument[]> {
  return request(async () =>
    DocumentListSchema.parse(await apiClient('/api/documents', { query: owner }))
  )
}

export async function createDocument(
  owner: DocumentOwner,
  input: { title: string; kind: string; bodyMarkdown: string }
): Promise<RekallDocument> {
  return request(async () =>
    DocumentSchema.parse(
      await apiClient('/api/documents', { method: 'POST', body: { ...input, ...owner } })
    )
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

export async function searchDocuments(query: string): Promise<RekallDocument[]> {
  return request(async () =>
    DocumentListSchema.parse(await apiClient('/api/documents/search', { query: { query } }))
  )
}
