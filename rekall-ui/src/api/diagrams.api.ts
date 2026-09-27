import { apiClient, request } from './client'
import {
  DiagramSchema,
  DiagramSummaryListSchema,
  SourceExcerptSchema
} from './schemas/diagram.schema'
import type { Diagram, DiagramDraft, DiagramSummary, SourceExcerpt, SourceLocation } from '@/model/diagram'
import type { DiagramId } from '@/model/branded'

export async function fetchDiagrams(): Promise<DiagramSummary[]> {
  return request(async () => DiagramSummaryListSchema.parse(await apiClient('/api/diagrams')))
}

export async function fetchDiagram(id: DiagramId): Promise<Diagram> {
  return request(async () => DiagramSchema.parse(await apiClient(`/api/diagrams/${id}`)))
}

export async function createDiagram(draft: DiagramDraft): Promise<Diagram> {
  return request(async () =>
    DiagramSchema.parse(await apiClient('/api/diagrams', { method: 'POST', body: draft }))
  )
}

export async function deleteDiagram(id: DiagramId): Promise<void> {
  await request(() => apiClient(`/api/diagrams/${id}`, { method: 'DELETE' }))
}

export async function fetchSourceExcerpt(id: DiagramId, source: SourceLocation): Promise<SourceExcerpt> {
  const query: Record<string, string> = { file: source.file }
  if (source.startLine !== null) query.start = String(source.startLine)
  if (source.endLine !== null) query.end = String(source.endLine)
  return request(async () =>
    SourceExcerptSchema.parse(await apiClient(`/api/diagrams/${id}/source`, { query }))
  )
}
