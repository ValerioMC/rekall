import { reactive, ref, watch, type Ref } from 'vue'
import { isEpic, type Task } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'
import { loadPersistedSet, savePersistedSet, toggledSet } from '@/common/config/persisted-set'

const COLLAPSE_KEY = 'rekall.nav.collapsed-projects'

/**
 * What the navigator has folded away: whole project cards, kept across restarts, and the epics,
 * backlog and filed shelves, per project when grouped and once for the list when not. A shelved
 * task picked from elsewhere (search, a note) opens its shelf so the selection is visible.
 */
export interface NavigatorShelves {
  readonly collapsedProjects: ReadonlySet<ProjectId>
  readonly openEpics: ReadonlySet<ProjectId>
  readonly openBacklogs: ReadonlySet<ProjectId>
  readonly openFiled: ReadonlySet<ProjectId>
  readonly epicsOpen: boolean
  readonly backlogOpen: boolean
  readonly filedOpen: boolean
  toggleCollapsed(projectId: ProjectId): void
  toggleEpics(projectId: ProjectId): void
  toggleBacklog(projectId: ProjectId): void
  toggleFiled(projectId: ProjectId): void
  toggleListEpics(): void
  toggleListBacklog(): void
  toggleListFiled(): void
}

export function useNavigatorShelves(selectedTask: Ref<Task | null>): NavigatorShelves {
  const collapsed = ref<ReadonlySet<ProjectId>>(loadPersistedSet<ProjectId>(COLLAPSE_KEY))
  const epics = ref<ReadonlySet<ProjectId>>(new Set())
  const backlogs = ref<ReadonlySet<ProjectId>>(new Set())
  const filed = ref<ReadonlySet<ProjectId>>(new Set())

  const shelves = reactive({
    collapsedProjects: collapsed,
    openEpics: epics,
    openBacklogs: backlogs,
    openFiled: filed,
    epicsOpen: false,
    backlogOpen: false,
    filedOpen: false,
    toggleCollapsed(projectId: ProjectId): void {
      collapsed.value = toggledSet(collapsed.value, projectId)
      savePersistedSet(COLLAPSE_KEY, collapsed.value)
    },
    toggleEpics(projectId: ProjectId): void {
      epics.value = toggledSet(epics.value, projectId)
    },
    toggleBacklog(projectId: ProjectId): void {
      backlogs.value = toggledSet(backlogs.value, projectId)
    },
    toggleFiled(projectId: ProjectId): void {
      filed.value = toggledSet(filed.value, projectId)
    },
    toggleListEpics(): void {
      shelves.epicsOpen = !shelves.epicsOpen
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
      if (task && isEpic(task)) {
        shelves.epicsOpen = true
        if (!epics.value.has(task.projectId)) shelves.toggleEpics(task.projectId)
      } else if (task?.status === 'DONE') {
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
