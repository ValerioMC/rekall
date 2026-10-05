import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  createProjectSection,
  deleteProjectSection,
  fetchProjectSections,
  reorderProjectSections,
  updateProjectSection
} from '@/api/project-sections.api'
import { movedOne, type Direction } from '@/common/project/section-order'
import type { ProjectSection, ProjectSectionInput } from '@/model/project-section'
import type { ProjectId, ProjectSectionId } from '@/model/branded'

/**
 * The sections of the projects the console has opened, each list kept in the order shown. A list
 * is fetched when its project is opened and again whenever the event stream says it changed.
 */
export const useProjectSectionsStore = defineStore('projectSections', () => {
  const byProject = ref<Readonly<Record<string, readonly ProjectSection[]>>>({})
  const loading = ref<ReadonlySet<string>>(new Set())

  function sectionsOf(projectId: ProjectId): readonly ProjectSection[] {
    return byProject.value[projectId] ?? []
  }

  function isLoaded(projectId: ProjectId): boolean {
    return projectId in byProject.value
  }

  function isLoading(projectId: ProjectId): boolean {
    return loading.value.has(projectId)
  }

  function put(projectId: ProjectId, sections: readonly ProjectSection[]): void {
    byProject.value = { ...byProject.value, [projectId]: sections }
  }

  async function load(projectId: ProjectId): Promise<void> {
    loading.value = new Set(loading.value).add(projectId)
    try {
      put(projectId, await fetchProjectSections(projectId))
    } finally {
      const rest = new Set(loading.value)
      rest.delete(projectId)
      loading.value = rest
    }
  }

  async function create(projectId: ProjectId, title: string): Promise<ProjectSection> {
    const created = await createProjectSection(projectId, { title, bodyMarkdown: '' })
    put(projectId, [...sectionsOf(projectId), created])
    return created
  }

  async function update(id: ProjectSectionId, input: ProjectSectionInput): Promise<ProjectSection> {
    const saved = await updateProjectSection(id, input)
    put(
      saved.projectId,
      sectionsOf(saved.projectId).map((known) => (known.id === saved.id ? saved : known))
    )
    return saved
  }

  async function move(projectId: ProjectId, id: ProjectSectionId, direction: Direction): Promise<void> {
    const before = sectionsOf(projectId)
    const ids = movedOne(before.map((section) => section.id), id, direction)
    // Shown at once; the server's answer, or a reload on failure, has the last word.
    put(projectId, ids.flatMap((moved) => before.filter((section) => section.id === moved)))
    try {
      put(projectId, await reorderProjectSections(projectId, ids))
    } catch (error) {
      put(projectId, before)
      throw error
    }
  }

  async function remove(projectId: ProjectId, id: ProjectSectionId): Promise<void> {
    await deleteProjectSection(id)
    put(projectId, sectionsOf(projectId).filter((section) => section.id !== id))
  }

  /** A change made elsewhere (a session, another window): read that project's list again if it is open here. */
  async function applyEvent(projectId: ProjectId): Promise<void> {
    if (isLoaded(projectId)) await load(projectId)
  }

  return { sectionsOf, isLoaded, isLoading, load, create, update, move, remove, applyEvent }
})
