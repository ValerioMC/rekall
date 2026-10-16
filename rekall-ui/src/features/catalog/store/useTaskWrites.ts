import {
  createTask as apiCreateTask,
  deleteTask as apiDeleteTask,
  reviewTask as apiReviewTask,
  updateTask as apiUpdateTask
} from '@/features/catalog/api/catalog.api'
import type { TaskInput } from '@/features/catalog/api/catalog.api'
import type { Task, TaskReview, TaskStatus } from '@/features/catalog/model/catalog'
import type { TagId, TaskId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'
import type { useConsoleNavigation } from '../../../stores/console/useConsoleNavigation'
import type { useConsoleRefreshes } from '../../../stores/console/useConsoleRefreshes'
import type { useTimers } from '../../time/store/useTimers'
import { blankToNull, replaceById, trackSave } from '../../../stores/console/rows'

/** The task as stored, with one field changed, in the shape its update takes. */
function taskInput(
  task: Task,
  change: Partial<Pick<TaskInput, 'status' | 'description' | 'tagId'>>
): TaskInput {
  return {
    label: task.label,
    title: task.title,
    status: task.status,
    description: task.description,
    projectId: task.projectId,
    tagId: task.tagId,
    ...change
  }
}

/** Writes on a task from the console. Working on a task that is not parked resumes its timer. */
export function useTaskWrites(
  state: ConsoleState,
  navigation: ReturnType<typeof useConsoleNavigation>,
  refreshes: ReturnType<typeof useConsoleRefreshes>,
  timers: ReturnType<typeof useTimers>
) {
  const { tasks, wrapups, steps, selectedTaskId, selectedDocId, paneFocus, saveState } = state

  function storeTask(saved: Task): void {
    tasks.value = replaceById(tasks.value, saved)
  }

  async function createTask(input: TaskInput): Promise<Task> {
    const created = await apiCreateTask(input)
    tasks.value = [...tasks.value, created]
    await Promise.all([refreshes.refreshProjects(), refreshes.refreshCompanies()])
    navigation.selectTask(created.id)
    // A task is created without its description, so it opens where that is written.
    paneFocus.value = 'description'
    // Filing a task in the backlog parks it: nothing is being worked on yet.
    if (created.status !== 'BACKLOG') await timers.startTimer(created.id)
    return created
  }

  async function updateTask(id: TaskId, input: TaskInput): Promise<void> {
    const saved = await apiUpdateTask(id, input)
    storeTask(saved)
    await Promise.all([
      refreshes.refreshDocuments(),
      refreshes.refreshProjects(),
      refreshes.refreshCompanies(),
      refreshes.refreshWrapups(),
      refreshes.refreshTimeEntries()
    ])
    if (saved.status !== 'DONE' && saved.status !== 'BACKLOG') await timers.resumeTimerOnWork(id)
  }

  async function deleteTask(id: TaskId): Promise<void> {
    await apiDeleteTask(id)
    tasks.value = tasks.value.filter((task) => task.id !== id)
    wrapups.value = wrapups.value.filter((wrapup) => wrapup.taskId !== id)
    steps.value = steps.value.filter((step) => step.taskId !== id)
    if (selectedTaskId.value === id) {
      selectedTaskId.value = null
      selectedDocId.value = null
      paneFocus.value = 'note'
    }
    await Promise.all([
      refreshes.refreshDocuments(),
      refreshes.refreshProjects(),
      refreshes.refreshCompanies()
    ])
  }

  async function setTaskStatus(id: TaskId, status: TaskStatus): Promise<void> {
    const task = tasks.value.find((candidate) => candidate.id === id)
    if (!task || task.status === status) return
    storeTask(await apiUpdateTask(id, taskInput(task, { status })))
    if (status === 'DONE') await refreshes.refreshTimeEntries()
  }

  async function setTaskTag(id: TaskId, tagId: TagId | null): Promise<void> {
    const task = tasks.value.find((candidate) => candidate.id === id)
    if (!task || task.tagId === tagId) return
    storeTask(await apiUpdateTask(id, taskInput(task, { tagId })))
  }

  async function saveTaskDescription(id: TaskId, description: string): Promise<void> {
    const task = tasks.value.find((candidate) => candidate.id === id)
    if (!task) return
    const next = blankToNull(description)
    if (task.description === next) return
    await trackSave(saveState, async () =>
      storeTask(await apiUpdateTask(id, taskInput(task, { description: next })))
    )
    await timers.resumeTimerOnWork(id)
  }

  function applyTaskReview(review: TaskReview): void {
    tasks.value = tasks.value.map((task) =>
      task.id === review.taskId
        ? {
            ...task,
            reviewState: review.reviewState,
            reviewActive: review.reviewActive,
            claimedAt: review.claimedAt,
            acceptedAt: review.acceptedAt,
            reviewNote: review.reviewNote
          }
        : task
    )
  }

  async function acceptTask(id: TaskId): Promise<void> {
    storeTask(await apiReviewTask(id, 'DONE'))
    await timers.rereadTimerStoppedByAccept(id)
  }

  async function sendBackTask(id: TaskId, note?: string): Promise<void> {
    storeTask(await apiReviewTask(id, 'OPEN', note ?? null))
  }

  /** A session created a task. Nothing is selected: the reader stays where they were. */
  async function applyTaskCreated(): Promise<void> {
    await Promise.all([refreshes.refreshTasks(), refreshes.refreshProjects(), refreshes.refreshCompanies()])
  }

  return {
    createTask,
    updateTask,
    deleteTask,
    setTaskStatus,
    setTaskTag,
    saveTaskDescription,
    applyTaskReview,
    applyTaskCreated,
    acceptTask,
    sendBackTask
  }
}
