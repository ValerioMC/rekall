import { watch } from 'vue'
import { useRoute } from 'vue-router'
import { projectTabOf } from '@/features/project/model/project-page'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import { useConsoleStore } from '@/stores/console.store'

/**
 * Fills the working set from what the person does: a task page opens that task, and a task page or
 * a project's task list is the place last seen in its project. Each place is also noted as the
 * one the flip key goes back from. Leaving a page closes nothing, and
 * nothing a session or the queue does is read here.
 */
export function useWorkingSetFollowsRoute(): void {
  const route = useRoute()
  const catalog = useConsoleStore()
  const workingSet = useWorkingSetStore()

  function follow(): void {
    if (catalog.isLoading || !workingSet.isLoaded) return
    if (route.name === 'task') visitTask(String(route.params.id))
    else if (route.name === 'project-detail') visitTaskList(String(route.params.id), route.params.tab)
  }

  function visitTask(id: string): void {
    const task = catalog.tasks.find((candidate) => candidate.id === id)
    if (!task) return
    workingSet.noteVisit({ projectId: task.projectId, taskId: task.id })
    void workingSet.open(task.id, task.projectId)
    void workingSet.remember({ projectId: task.projectId, taskId: task.id })
  }

  function visitTaskList(id: string, tab: unknown): void {
    const project = catalog.projects.find((candidate) => candidate.id === id)
    if (!project || projectTabOf(typeof tab === 'string' ? tab : undefined) !== 'tasks') return
    workingSet.noteVisit({ projectId: project.id, taskId: null })
    void workingSet.remember({ projectId: project.id, taskId: null })
  }

  watch(() => [route.name, route.params.id, route.params.tab, catalog.isLoading, workingSet.isLoaded] as const, follow, { immediate: true })
}
