import { importJiraIssues as apiImportJiraIssues } from '@/features/connection/api/connections.api'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectConnectionId, ProjectId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'
import type { useConsoleRefreshes } from '../../../stores/console/useConsoleRefreshes'
import { upsertById } from '../../../stores/console/rows'

/** Jira issues imported as a project's tasks, which join the console's tasks as they are created. */
export function useJiraImport(state: ConsoleState, refreshes: ReturnType<typeof useConsoleRefreshes>) {
  const { tasks } = state

  async function importJiraIssues(
    projectId: ProjectId,
    bindingId: ProjectConnectionId,
    keys: readonly string[]
  ): Promise<Task[]> {
    const created = await apiImportJiraIssues(projectId, bindingId, keys)
    tasks.value = created.reduce((rows, task) => upsertById(rows, task, 'last'), tasks.value)
    await Promise.all([refreshes.refreshProjects(), refreshes.refreshCompanies()])
    return created
  }

  return { importJiraIssues }
}
