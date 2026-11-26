import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import { useTerminalPaneShown } from '@/features/claude/composables/useTerminalPaneShown'
import type { OpenPlace } from '@/features/workingset/model/working-set'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import { useConsoleStore } from '@/stores/console.store'
import type { ProjectId } from '@/model/branded'

/** Where the person is in the bar, and how to move to a place: shared by the bar and its keys. */
export function useWorkingSetNavigation() {
  const route = useRoute()
  const router = useRouter()
  const workingSet = useWorkingSetStore()
  const { tasks, selectedTask } = storeToRefs(useConsoleStore())
  const terminalShown = useTerminalPaneShown()

  const taskInView = computed(() => {
    if (terminalShown.value) return selectedTask.value
    return route.name === 'task' ? (tasks.value.find((task) => task.id === route.params.id) ?? null) : null
  })
  const projectIdInView = computed<ProjectId | null>(() => {
    if (taskInView.value) return taskInView.value.projectId
    return route.name === 'project-detail' ? (String(route.params.id) as ProjectId) : null
  })

  /** The place last seen in the project, or its task list when that task is no longer open. */
  function placeToVisit(projectId: ProjectId): OpenPlace {
    const remembered = workingSet.placeOf(projectId)
    return remembered.taskId !== null && workingSet.isOpen(remembered.taskId) ? remembered : { projectId, taskId: null }
  }

  function visit(place: OpenPlace): void {
    if (place.taskId === null) void router.push(`/projects/${place.projectId}`)
    else void router.push({ name: 'task', params: { id: place.taskId, tab: 'description' } })
  }

  return { taskInView, projectIdInView, placeToVisit, visit }
}
