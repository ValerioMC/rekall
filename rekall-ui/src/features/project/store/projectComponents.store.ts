import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  createProjectComponent,
  deleteProjectComponent,
  fetchProjectComponents,
  reorderProjectComponents,
  updateProjectComponent
} from '@/features/project/api/project-components.api'
import { movedOne, type Direction } from '@/features/project/lib/section-order'
import { launchChoices } from '@/common/config/claude-launch'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { Terminal } from '@/features/claude/model/terminal'
import type { ProjectComponent, ProjectComponentInput } from '@/features/project/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

/**
 * The components of the projects the console has opened, each list kept in the order shown. A
 * list is fetched when its project is opened and again whenever the event stream says it changed.
 */
export const useProjectComponentsStore = defineStore('projectComponents', () => {
  const byProject = ref<Readonly<Record<string, readonly ProjectComponent[]>>>({})
  const loading = ref<ReadonlySet<string>>(new Set())

  function componentsOf(projectId: ProjectId): readonly ProjectComponent[] {
    return byProject.value[projectId] ?? []
  }

  /** Whether a list has arrived, so an empty project is told from one not fetched yet. */
  function isLoaded(projectId: ProjectId): boolean {
    return projectId in byProject.value
  }

  function isLoading(projectId: ProjectId): boolean {
    return loading.value.has(projectId)
  }

  function put(projectId: ProjectId, components: readonly ProjectComponent[]): void {
    byProject.value = { ...byProject.value, [projectId]: components }
  }

  async function load(projectId: ProjectId): Promise<void> {
    loading.value = new Set(loading.value).add(projectId)
    try {
      put(projectId, await fetchProjectComponents(projectId))
    } finally {
      const rest = new Set(loading.value)
      rest.delete(projectId)
      loading.value = rest
    }
  }

  async function ensureLoaded(projectId: ProjectId): Promise<void> {
    if (!isLoaded(projectId) && !isLoading(projectId)) await load(projectId)
  }

  async function create(projectId: ProjectId, input: ProjectComponentInput): Promise<ProjectComponent> {
    const created = await createProjectComponent(projectId, input)
    put(projectId, [...componentsOf(projectId), created])
    return created
  }

  /** A session that reads the project's folder and writes each part of it as a component. */
  function scan(projectId: ProjectId): Promise<Terminal> {
    return useTerminalStore().openForProject(projectId, { ...launchChoices(), mode: 'SCAN_COMPONENTS' })
  }

  /** A change made elsewhere (a session, another window): read that project's list again if it is open here. */
  async function applyEvent(projectId: ProjectId): Promise<void> {
    if (isLoaded(projectId)) await load(projectId)
  }

  async function update(id: ProjectComponentId, input: ProjectComponentInput): Promise<ProjectComponent> {
    const saved = await updateProjectComponent(id, input)
    put(
      saved.projectId,
      componentsOf(saved.projectId).map((known) => (known.id === saved.id ? saved : known))
    )
    return saved
  }

  async function move(projectId: ProjectId, id: ProjectComponentId, direction: Direction): Promise<void> {
    const before = componentsOf(projectId)
    const ids = movedOne(before.map((component) => component.id), id, direction)
    // Shown at once; the server's answer, or the old order on failure, has the last word.
    put(projectId, ids.flatMap((moved) => before.filter((component) => component.id === moved)))
    try {
      put(projectId, await reorderProjectComponents(projectId, ids))
    } catch (error) {
      put(projectId, before)
      throw error
    }
  }

  async function remove(projectId: ProjectId, id: ProjectComponentId): Promise<void> {
    await deleteProjectComponent(id)
    put(projectId, componentsOf(projectId).filter((component) => component.id !== id))
  }

  return { componentsOf, isLoaded, isLoading, load, ensureLoaded, create, scan, update, move, remove, applyEvent }
})
