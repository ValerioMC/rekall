import { apiClient, request } from '../../../api/client'
import { env } from '@/common/config/env'
import { toBase64 } from '@/features/export/lib/base64'
import { ProjectExportListSchema, ProjectExportSchema } from './schemas/project-export.schema'
import type { ProjectExportEntry, ProjectExportRequest } from '@/features/export/model/project-export'
import type { ProjectExportId, ProjectId } from '@/model/branded'

/** Posted once and never retried: a retry after a server failure would record the failure again. */
export async function exportProject(projectId: ProjectId, input: ProjectExportRequest): Promise<ProjectExportEntry> {
  const body = {
    format: input.format.toLowerCase(),
    blueprint: input.blueprint,
    sectionIds: input.sectionIds,
    componentIds: input.componentIds,
    diagrams: input.diagrams.map((image) => ({ source: image.source, png: toBase64(image.png) }))
  }
  return request(async () => ProjectExportSchema.parse(await apiClient(`/api/projects/${projectId}/exports`, { method: 'POST', body, retry: 0 })))
}

export async function fetchExports(): Promise<ProjectExportEntry[]> {
  return request(async () => ProjectExportListSchema.parse(await apiClient('/api/exports')))
}

export async function deleteExport(id: ProjectExportId): Promise<void> {
  await request(() => apiClient(`/api/exports/${id}`, { method: 'DELETE' }))
}

export function exportFileUrl(id: ProjectExportId): string {
  return `${env.VITE_API_BASE_URL}/api/exports/${id}/file`
}
