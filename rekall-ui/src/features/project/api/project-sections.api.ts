import { apiClient, request } from '../../../api/client'
import {
  ProjectSectionListSchema,
  ProjectSectionSchema,
  SectionContextListSchema,
  SectionContextSchema,
  SectionLayoutSchema
} from './schemas/project-section.schema'
import type { ProjectSection, ProjectSectionInput } from '@/features/project/model/project-section'
import type { ContextPlacement, SectionContext, SectionLayout } from '@/features/project/model/section-context'
import type { ProjectId, ProjectSectionId, SectionContextId } from '@/model/branded'

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

export async function fetchSectionContexts(projectId: ProjectId): Promise<SectionContext[]> {
  return request(async () =>
    SectionContextListSchema.parse(await apiClient(`/api/projects/${projectId}/section-contexts`))
  )
}

export async function createSectionContext(projectId: ProjectId, title: string): Promise<SectionContext> {
  return request(async () =>
    SectionContextSchema.parse(
      await apiClient(`/api/projects/${projectId}/section-contexts`, { method: 'POST', body: { title } })
    )
  )
}

export async function renameSectionContext(id: SectionContextId, title: string): Promise<SectionContext> {
  return request(async () =>
    SectionContextSchema.parse(await apiClient(`/api/section-contexts/${id}`, { method: 'PUT', body: { title } }))
  )
}

export async function deleteSectionContext(id: SectionContextId): Promise<void> {
  await request(() => apiClient(`/api/section-contexts/${id}`, { method: 'DELETE' }))
}

export async function arrangeProjectSections(
  projectId: ProjectId,
  placements: readonly ContextPlacement[]
): Promise<SectionLayout> {
  return request(async () =>
    SectionLayoutSchema.parse(
      await apiClient(`/api/projects/${projectId}/sections/layout`, {
        method: 'PUT',
        body: { contexts: placements }
      })
    )
  )
}

export async function deleteProjectSection(id: ProjectSectionId): Promise<void> {
  await request(() => apiClient(`/api/project-sections/${id}`, { method: 'DELETE' }))
}
