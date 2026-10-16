import {
  createProject as apiCreateProject,
  deleteProject as apiDeleteProject,
  removeProjectIconImage as apiRemoveProjectIconImage,
  updateProject as apiUpdateProject,
  uploadProjectIconImage as apiUploadProjectIconImage
} from '@/features/catalog/api/catalog.api'
import type { ProjectInput } from '@/features/catalog/api/catalog.api'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'
import type { useConsoleNavigation } from '../../../stores/console/useConsoleNavigation'
import type { useConsoleRefreshes } from '../../../stores/console/useConsoleRefreshes'
import { blankToNull, replaceById, trackSave } from '../../../stores/console/rows'

type ProjectPagePatch = Partial<
  Pick<ProjectInput, 'description' | 'blueprintMarkdown' | 'repoFolder' | 'autoCommit'>
>

/** The record dialog writes a project whole; its own page saves one field at a time over what is stored. */
export function useProjectWrites(
  state: ConsoleState,
  navigation: ReturnType<typeof useConsoleNavigation>,
  refreshes: ReturnType<typeof useConsoleRefreshes>
) {
  const { projects, tasks, scopeCompany, scopeProject, saveState } = state

  async function createProject(input: ProjectInput): Promise<Project> {
    const created = await apiCreateProject(input)
    projects.value = [...projects.value, created].sort((a, b) => a.label.localeCompare(b.label))
    await refreshes.refreshCompanies()
    navigation.setScope(input.companyId, created.id)
    return created
  }

  async function updateProject(id: ProjectId, input: ProjectInput): Promise<void> {
    const saved = await apiUpdateProject(id, input)
    projects.value = replaceById(projects.value, saved).sort((a, b) => a.label.localeCompare(b.label))
    await Promise.all([
      refreshes.refreshTasks(),
      refreshes.refreshCompanies(),
      refreshes.refreshDocuments(),
      refreshes.refreshWrapups()
    ])
  }

  /** `null` takes the picture off, and the project goes back to its icon key. */
  async function setProjectIconImage(id: ProjectId, file: File | null): Promise<void> {
    if (file) {
      projects.value = replaceById(projects.value, await apiUploadProjectIconImage(id, file))
      return
    }
    await apiRemoveProjectIconImage(id)
    projects.value = projects.value.map((project) =>
      project.id === id ? { ...project, iconImageVersion: null } : project
    )
  }

  async function deleteProject(id: ProjectId): Promise<void> {
    await apiDeleteProject(id)
    if (scopeProject.value === id) navigation.setScope(scopeCompany.value)
    await refreshes.load()
  }

  /** Tasks carry their project's folder, so a saved folder is copied onto them. */
  async function patchProject(id: ProjectId, patch: ProjectPagePatch): Promise<void> {
    const current = projects.value.find((project) => project.id === id)
    if (!current) return
    await trackSave(saveState, async () => {
      const saved = await apiUpdateProject(id, {
        label: current.label,
        title: current.title,
        status: current.status,
        icon: current.icon,
        companyId: current.companyId,
        description: 'description' in patch ? patch.description! : current.description,
        blueprintMarkdown:
          'blueprintMarkdown' in patch ? patch.blueprintMarkdown! : current.blueprintMarkdown,
        repoFolder: 'repoFolder' in patch ? patch.repoFolder! : current.repoFolder,
        autoCommit: 'autoCommit' in patch ? patch.autoCommit! : current.autoCommit
      })
      projects.value = replaceById(projects.value, saved)
      tasks.value = tasks.value.map((task) =>
        task.projectId === id ? { ...task, projectRepoFolder: saved.repoFolder } : task
      )
    })
  }

  function saveProjectDescription(id: ProjectId, description: string): Promise<void> {
    return patchProject(id, { description: blankToNull(description) })
  }

  function saveProjectBlueprint(id: ProjectId, blueprintMarkdown: string): Promise<void> {
    return patchProject(id, { blueprintMarkdown: blankToNull(blueprintMarkdown) })
  }

  function saveProjectRepoFolder(id: ProjectId, repoFolder: string): Promise<void> {
    return patchProject(id, { repoFolder: blankToNull(repoFolder.trim()) })
  }

  // The server keeps it off outside a git repository, whatever is asked: the saved row is what counts.
  function saveProjectAutoCommit(id: ProjectId, autoCommit: boolean): Promise<void> {
    return patchProject(id, { autoCommit })
  }

  return {
    createProject,
    updateProject,
    deleteProject,
    setProjectIconImage,
    saveProjectDescription,
    saveProjectBlueprint,
    saveProjectRepoFolder,
    saveProjectAutoCommit
  }
}
