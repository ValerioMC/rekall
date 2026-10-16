import { reactive, ref, watch, type Ref } from 'vue'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'
import { loadPersistedSet, savePersistedSet, toggledSet } from '@/common/config/persisted-set'

const COLLAPSE_KEY = 'rekall.nav.collapsed-projects'

/**
 * What the navigator has folded away: whole project cards, kept across restarts, and the backlog
 * and filed shelves, per project when grouped and once for the list when not. A shelved task
 * picked from elsewhere (search, a note) opens its shelf so the selection is visible.
 */
export interface NavigatorShelves {
  readonly collapsedProjects: ReadonlySet<ProjectId>
  readonly openBacklogs: ReadonlySet<ProjectId>
  readonly openFiled: ReadonlySet<ProjectId>
  readonly backlogOpen: boolean
  readonly filedOpen: boolean
  toggleCollapsed(projectId: ProjectId): void
  toggleBacklog(projectId: ProjectId): void
  toggleFiled(projectId: ProjectId): void
  toggleListBacklog(): void
  toggleListFiled(): void
}

export function useNavigatorShelves(selectedTask: Ref<Task | null>): NavigatorShelves {
  const collapsed = ref<ReadonlySet<ProjectId>>(loadPersistedSet<ProjectId>(COLLAPSE_KEY))
  const backlogs = ref<ReadonlySet<ProjectId>>(new Set())
  const filed = ref<ReadonlySet<ProjectId>>(new Set())

  const shelves = reactive({
    collapsedProjects: collapsed,
    openBacklogs: backlogs,
    openFiled: filed,
    backlogOpen: false,
    filedOpen: false,
    toggleCollapsed(projectId: ProjectId): void {
      collapsed.value = toggledSet(collapsed.value, projectId)
      savePersistedSet(COLLAPSE_KEY, collapsed.value)
    },
    toggleBacklog(projectId: ProjectId): void {
      backlogs.value = toggledSet(backlogs.value, projectId)
    },
    toggleFiled(projectId: ProjectId): void {
      filed.value = toggledSet(filed.value, projectId)
    },
    toggleListBacklog(): void {
      shelves.backlogOpen = !shelves.backlogOpen
    },
    toggleListFiled(): void {
      shelves.filedOpen = !shelves.filedOpen
    }
  })

  watch(
    () => selectedTask.value?.id,
    () => {
      const task = selectedTask.value
      if (task?.status === 'DONE') {
        shelves.filedOpen = true
        if (!filed.value.has(task.projectId)) shelves.toggleFiled(task.projectId)
      } else if (task?.status === 'BACKLOG') {
        shelves.backlogOpen = true
        if (!backlogs.value.has(task.projectId)) shelves.toggleBacklog(task.projectId)
      }
    }
  )

  return shelves
}
