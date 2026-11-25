import { computed } from 'vue'
import { taskRows } from '@/features/task/lib/task-rows'
import { useConsoleStore } from '@/stores/console.store'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import type { TaskRow } from '@/features/task/model/task-row'
import type { TokenTotals } from '@/features/tokens/model/token-usage'
import type { ProjectId } from '@/model/branded'

export interface ProjectWork {
  rowsOf(projectId: ProjectId): readonly TaskRow[]
  tokensOf(projectId: ProjectId): TokenTotals
}

/** What each project's tasks look like and what they cost, for the lists that show projects as a ledger. */
export function useProjectWork(): ProjectWork {
  const store = useConsoleStore()
  const tokenUsage = useTokenUsageStore()

  const rowsByProject = computed(() => {
    const byProject = new Map<ProjectId, TaskRow[]>()
    for (const row of taskRows(store.tasks, store.steps, () => 0)) {
      byProject.set(row.task.projectId, [...(byProject.get(row.task.projectId) ?? []), row])
    }
    return byProject
  })

  return {
    rowsOf: (projectId) => rowsByProject.value.get(projectId) ?? [],
    tokensOf: (projectId) => tokenUsage.ofProject(projectId, store.tasks)
  }
}
