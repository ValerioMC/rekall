import { importConfluenceSpace as apiImportConfluenceSpace } from '@/features/connection/api/connections.api'
import type { ConfluenceImport } from '@/features/connection/model/connection'
import type { ProjectConnectionId, ProjectId } from '@/model/branded'
import type { useConsoleRefreshes } from '../../../stores/console/useConsoleRefreshes'

/** A project filled from a Confluence space, whose description and blueprint the console then reads again. */
export function useConfluenceImport(refreshes: ReturnType<typeof useConsoleRefreshes>) {
  async function importConfluenceSpace(
    projectId: ProjectId,
    bindingId: ProjectConnectionId,
    replace: boolean
  ): Promise<ConfluenceImport> {
    const imported = await apiImportConfluenceSpace(projectId, bindingId, replace)
    await refreshes.refreshProjects()
    return imported
  }

  return { importConfluenceSpace }
}
