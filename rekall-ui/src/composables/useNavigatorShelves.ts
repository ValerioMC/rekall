import { reactive, ref, watch, type Ref } from 'vue'
import { isEpic, type Task } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'
import { loadPersistedSet, savePersistedSet, toggledSet } from '@/common/config/persisted-set'

const COLLAPSE_KEY = 'rekall.nav.collapsed-projects'

/**
 * What the navigator has folded away: whole project cards, kept across restarts, the backlog and
 * filed shelves, per project when grouped and once for the list when not, and the epics that are
 * open (all start closed). A task picked from elsewhere (search, a note) opens what hides it: its
 * shelf, and the epic it belongs to.
 */
export interface NavigatorShelves {
  readonly collapsedProjects: ReadonlySet<ProjectId>
  readonly openBacklogs: ReadonlySet<ProjectId>
  readonly openFiled: ReadonlySet<ProjectId>
  readonly openEpics: ReadonlySet<TaskId>
  readonly backlogOpen: boolean
  readonly filedOpen: boolean
  toggleCollapsed(projectId: ProjectId): void
  toggleBacklog(projectId: ProjectId): void
  toggleFiled(projectId: ProjectId): void
  toggleEpic(epicId: TaskId): void
  toggleListBacklog(): void
  toggleListFiled(): void
}

export function useNavigatorShelves(
  selectedTask: Ref<Task | null>,
  tasks: Ref<readonly Task[]> = ref([])
): NavigatorShelves {
  const collapsed = ref<ReadonlySet<ProjectId>>(loadPersistedSet<ProjectId>(COLLAPSE_KEY))
  const backlogs = ref<ReadonlySet<ProjectId>>(new Set())
  const filed = ref<ReadonlySet<ProjectId>>(new Set())
  const epics = ref<ReadonlySet<TaskId>>(new Set())

  const shelves = reactive({
    collapsedProjects: collapsed,
    openBacklogs: backlogs,
    openFiled: filed,
    openEpics: epics,
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
    toggleEpic(epicId: TaskId): void {
      epics.value = toggledSet(epics.value, epicId)
    },
    toggleListBacklog(): void {
      shelves.backlogOpen = !shelves.backlogOpen
    },
    toggleListFiled(): void {
      shelves.filedOpen = !shelves.filedOpen
    }
  })

  function openShelfOf(task: Task): void {
    if (task.status === 'DONE') {
      shelves.filedOpen = true
      if (!filed.value.has(task.projectId)) shelves.toggleFiled(task.projectId)
    } else if (task.status === 'BACKLOG') {
      shelves.backlogOpen = true
      if (!backlogs.value.has(task.projectId)) shelves.toggleBacklog(task.projectId)
    }
  }

  watch(
    () => selectedTask.value?.id,
    () => {
      const task = selectedTask.value
      if (!task) return
      const epic = isEpic(task) ? null : (tasks.value.find((candidate) => candidate.id === task.epicId) ?? null)
      if (epic && !epics.value.has(epic.id)) shelves.toggleEpic(epic.id)
      openShelfOf(epic ?? task)
    }
  )

  return shelves
}
