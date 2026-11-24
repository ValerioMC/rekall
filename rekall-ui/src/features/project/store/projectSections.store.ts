import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  arrangeProjectSections,
  createProjectSection,
  createSectionContext,
  deleteProjectSection,
  deleteSectionContext,
  fetchProjectSections,
  fetchSectionContexts,
  renameSectionContext,
  updateProjectSection
} from '@/features/project/api/project-sections.api'
import { arranged } from '@/features/project/lib/section-layout'
import { launchChoices } from '@/common/config/session-launch'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { ProjectSection, ProjectSectionInput } from '@/features/project/model/project-section'
import type { ContextPlacement, SectionContext } from '@/features/project/model/section-context'
import type { ProjectId, ProjectSectionId, SectionContextId } from '@/model/branded'

/**
 * The sections and contexts of the projects the console has opened, each list kept in the order
 * shown. A project is fetched when it is opened and again whenever the event stream says it changed.
 */
export const useProjectSectionsStore = defineStore('projectSections', () => {
  const byProject = ref<Readonly<Record<string, readonly ProjectSection[]>>>({})
  const contextsByProject = ref<Readonly<Record<string, readonly SectionContext[]>>>({})
  const loading = ref<ReadonlySet<string>>(new Set())

  function sectionsOf(projectId: ProjectId): readonly ProjectSection[] {
    return byProject.value[projectId] ?? []
  }

  function contextsOf(projectId: ProjectId): readonly SectionContext[] {
    return contextsByProject.value[projectId] ?? []
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

  function putContexts(projectId: ProjectId, contexts: readonly SectionContext[]): void {
    contextsByProject.value = { ...contextsByProject.value, [projectId]: contexts }
  }

  async function load(projectId: ProjectId): Promise<void> {
    loading.value = new Set(loading.value).add(projectId)
    try {
      const [sections, contexts] = await Promise.all([fetchProjectSections(projectId), fetchSectionContexts(projectId)])
      put(projectId, sections)
      putContexts(projectId, contexts)
    } finally {
      const rest = new Set(loading.value)
      rest.delete(projectId)
      loading.value = rest
    }
  }

  async function create(
    projectId: ProjectId,
    title: string,
    bodyMarkdown = '',
    contextId?: SectionContextId
  ): Promise<ProjectSection> {
    const input: ProjectSectionInput = contextId ? { title, bodyMarkdown, contextId } : { title, bodyMarkdown }
    const created = await createProjectSection(projectId, input)
    put(projectId, [...sectionsOf(projectId), created])
    if (!contextsOf(projectId).some((context) => context.id === created.contextId)) {
      putContexts(projectId, await fetchSectionContexts(projectId))
    }
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

  /** Puts every context and page where `placements` says; shown at once, and the server's answer, or the old arrangement on failure, has the last word. */
  async function arrange(projectId: ProjectId, placements: readonly ContextPlacement[]): Promise<void> {
    const sectionsBefore = sectionsOf(projectId)
    const contextsBefore = contextsOf(projectId)
    const shown = arranged(contextsBefore, sectionsBefore, placements)
    put(projectId, shown.sections)
    putContexts(projectId, shown.contexts)
    try {
      const saved = await arrangeProjectSections(projectId, placements)
      put(projectId, saved.sections)
      putContexts(projectId, saved.contexts)
    } catch (error) {
      put(projectId, sectionsBefore)
      putContexts(projectId, contextsBefore)
      throw error
    }
  }

  async function remove(projectId: ProjectId, id: ProjectSectionId): Promise<void> {
    await deleteProjectSection(id)
    put(projectId, sectionsOf(projectId).filter((section) => section.id !== id))
  }

  async function createContext(projectId: ProjectId, title: string): Promise<SectionContext> {
    const created = await createSectionContext(projectId, title)
    putContexts(projectId, [...contextsOf(projectId), created])
    return created
  }

  async function renameContext(id: SectionContextId, title: string): Promise<SectionContext> {
    const saved = await renameSectionContext(id, title)
    putContexts(
      saved.projectId,
      contextsOf(saved.projectId).map((known) => (known.id === saved.id ? saved : known))
    )
    return saved
  }

  async function removeContext(projectId: ProjectId, id: SectionContextId): Promise<void> {
    await deleteSectionContext(id)
    putContexts(projectId, contextsOf(projectId).filter((context) => context.id !== id))
  }

  /** A change made elsewhere (a session, another window): read that project's lists again if it is open here. */
  async function applyEvent(projectId: ProjectId): Promise<void> {
    if (isLoaded(projectId)) await load(projectId)
  }

  /** A session that learns from the project's repository how it takes a contribution, and writes that as sections. */
  function autoconfigure(projectId: ProjectId): Promise<Terminal> {
    return useTerminalStore().openForProject(projectId, { ...launchChoices(), mode: 'AUTOCONFIGURE' })
  }

  /** A session that reads every page and regroups them in contexts by concept. */
  function reorganize(projectId: ProjectId): Promise<Terminal> {
    return useTerminalStore().openForProject(projectId, { ...launchChoices(), mode: 'REORGANIZE' })
  }

  return {
    sectionsOf,
    contextsOf,
    isLoaded,
    isLoading,
    load,
    create,
    update,
    arrange,
    remove,
    createContext,
    renameContext,
    removeContext,
    applyEvent,
    autoconfigure,
    reorganize
  }
})
