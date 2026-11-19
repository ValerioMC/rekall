import type { SubNavItem } from '@/components/ui/SubNav.vue'
import { TASK_TAB_LABEL, TASK_TABS, type TaskTab } from '@/features/task/model/task-tab'
import type { TaskId } from '@/model/branded'

export interface TaskTabCounts {
  readonly steps: number
  readonly claimedSteps: number
  readonly notes: number
  readonly deviations: number
  readonly commits: number
}

function countOf(tab: TaskTab, counts: TaskTabCounts): number | undefined {
  if (tab === 'plan') return counts.steps
  if (tab === 'notes') return counts.notes
  if (tab === 'deviations') return counts.deviations
  if (tab === 'changes') return counts.commits
  return undefined
}

/** Tabs as links to the task's own routes; an epic has no plan and changes nothing itself. A count shows only where it means something, and Plan's turns amber while a step waits. */
export function taskTabItems(taskId: TaskId, counts: TaskTabCounts, isEpic = false): readonly SubNavItem<TaskTab>[] {
  return TASK_TABS.filter((tab) => !(isEpic && (tab === 'plan' || tab === 'changes'))).map((tab) => {
    const count = countOf(tab, counts)
    return {
      id: tab,
      label: TASK_TAB_LABEL[tab],
      to: `/task/${taskId}/${tab}`,
      ...(count !== undefined && count > 0 ? { count } : {}),
      ...(tab === 'plan' && counts.claimedSteps > 0 ? { hot: true } : {})
    }
  })
}
