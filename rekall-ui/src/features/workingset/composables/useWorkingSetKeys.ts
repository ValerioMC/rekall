import { onMounted, onUnmounted } from 'vue'
import { useModalGate } from '@/composables/useModalGate'
import { useWorkingSetNavigation } from '@/features/workingset/composables/useWorkingSetNavigation'
import { stepThroughTasks } from '@/features/workingset/lib/step-through-tasks'
import type { OpenPlace } from '@/features/workingset/model/working-set'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'

const PROJECT_KEY = /^[1-9]$/

/**
 * `1` to `9` open the n-th project of the bar, `[` and `]` step through the open tasks and the
 * backtick flips to the previous place. Bare keys, like the console's others: nothing while a
 * dialog is open or a field has the keyboard, and nothing with Cmd, Ctrl or Alt held.
 */
export function useWorkingSetKeys(): void {
  const workingSet = useWorkingSetStore()
  const { taskInView, placeToVisit, visit } = useWorkingSetNavigation()
  const { isModalOpen } = useModalGate()

  function projectIds() {
    return workingSet.groups.flatMap((company) => company.projects.map((group) => group.project.id))
  }

  function openTaskIds() {
    return workingSet.groups.flatMap((company) => company.projects.flatMap((group) => group.tasks.map((task) => task.id)))
  }

  function stepTask(direction: 1 | -1): void {
    const target = stepThroughTasks(openTaskIds(), taskInView.value?.id ?? null, direction)
    const task = workingSet.groups.flatMap((company) => company.projects.flatMap((group) => group.tasks)).find((candidate) => candidate.id === target)
    if (task) visit({ projectId: task.projectId, taskId: task.id })
  }

  function flip(): void {
    const previous = workingSet.previousPlace
    if (!previous) return
    const stillOpen = previous.taskId === null || workingSet.isOpen(previous.taskId)
    const place: OpenPlace = stillOpen ? previous : { projectId: previous.projectId, taskId: null }
    visit(place)
  }

  function pickProject(key: string): void {
    const projectId = projectIds()[Number(key) - 1]
    if (projectId) visit(placeToVisit(projectId))
  }

  function onKeydown(event: KeyboardEvent): void {
    if (isModalOpen.value > 0 || event.metaKey || event.ctrlKey || event.altKey || isTyping(event.target)) return
    if (PROJECT_KEY.test(event.key)) pickProject(event.key)
    else if (event.key === '[') stepTask(-1)
    else if (event.key === ']') stepTask(1)
    else if (event.key === '`') flip()
    else return
    event.preventDefault()
  }

  onMounted(() => window.addEventListener('keydown', onKeydown))
  onUnmounted(() => window.removeEventListener('keydown', onKeydown))
}

function isTyping(target: EventTarget | null): boolean {
  const element = target as HTMLElement | null
  return !!element && (/^(INPUT|TEXTAREA|SELECT)$/.test(element.tagName) || element.isContentEditable)
}
