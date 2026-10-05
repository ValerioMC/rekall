import { apiClient, request } from './client'
import { ProjectSectionListSchema, ProjectSectionSchema } from './schemas/project-section.schema'
import type { ProjectSection, ProjectSectionInput } from '@/model/project-section'
import type { ProjectId, ProjectSectionId } from '@/model/branded'

export async function fetchProjectSections(projectId: ProjectId): Promise<ProjectSection[]> {
  return request(async () =>
    ProjectSectionListSchema.parse(await apiClient(`/api/projects/${projectId}/sections`))
  )
}

export async function createProjectSection(
  projectId: ProjectId,
  input: ProjectSectionInput
): Promise<ProjectSection> {
  return request(async () =>
    ProjectSectionSchema.parse(
      await apiClient(`/api/projects/${projectId}/sections`, { method: 'POST', body: input })
    )
  )
}

export async function updateProjectSection(
  id: ProjectSectionId,
  input: ProjectSectionInput
): Promise<ProjectSection> {
  return request(async () =>
    ProjectSectionSchema.parse(
      await apiClient(`/api/project-sections/${id}`, { method: 'PUT', body: input })
    )
  )
}

export async function reorderProjectSections(
  projectId: ProjectId,
  sectionIds: readonly ProjectSectionId[]
): Promise<ProjectSection[]> {
  return request(async () =>
    ProjectSectionListSchema.parse(
      await apiClient(`/api/projects/${projectId}/sections/order`, {
        method: 'PUT',
        body: { sectionIds }
      })
    )
  )
}

export async function deleteProjectSection(id: ProjectSectionId): Promise<void> {
  await request(() => apiClient(`/api/project-sections/${id}`, { method: 'DELETE' }))
}
