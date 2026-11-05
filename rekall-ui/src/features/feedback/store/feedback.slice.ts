import { computed, ref } from 'vue'
import {
  addFeedback as apiAddFeedback,
  deleteFeedback as apiDeleteFeedback,
  discardRevision as apiDiscardRevision,
  editFeedback as apiEditFeedback,
  fetchFeedback
} from '@/features/feedback/api/feedback.api'
import { isPending, type Feedback, type FeedbackKind, type WrittenFeedbackKind } from '@/features/feedback/model/feedback'
import type { TaskId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'
import type { useConsoleRefreshes } from '../../../stores/console/useConsoleRefreshes'
import { replaceById, trackSave, upsertById } from '../../../stores/console/rows'

/** The thread under each task's description: read when the task is shown and whenever the server says it changed. */
export function useFeedbackThread(state: ConsoleState, refreshes: ReturnType<typeof useConsoleRefreshes>) {
  const { selectedTaskId, saveState } = state
  const feedbackByTask = ref<Readonly<Record<string, readonly Feedback[]>>>({})

  const selectedTaskFeedback = computed<readonly Feedback[]>(() =>
    selectedTaskId.value === null ? [] : (feedbackByTask.value[selectedTaskId.value] ?? [])
  )

  function pendingCount(kind: FeedbackKind): number {
    return selectedTaskFeedback.value.filter((entry) => entry.kind === kind && isPending(entry)).length
  }

  function putThread(taskId: TaskId, thread: readonly Feedback[]): void {
    feedbackByTask.value = { ...feedbackByTask.value, [taskId]: thread }
  }

  function threadOf(taskId: TaskId): readonly Feedback[] {
    return feedbackByTask.value[taskId] ?? []
  }

  async function loadFeedback(taskId: TaskId): Promise<void> {
    putThread(taskId, await fetchFeedback(taskId))
  }

  async function addFeedback(taskId: TaskId, kind: WrittenFeedbackKind, bodyMarkdown: string): Promise<void> {
    const added = await trackSave(saveState, () => apiAddFeedback(taskId, kind, bodyMarkdown))
    putThread(taskId, upsertById(threadOf(taskId), added, 'last'))
  }

  async function editFeedback(entry: Feedback, bodyMarkdown: string): Promise<void> {
    const edited = await trackSave(saveState, () => apiEditFeedback(entry.id, bodyMarkdown))
    putThread(entry.taskId, replaceById(threadOf(entry.taskId), edited))
  }

  async function deleteFeedback(entry: Feedback): Promise<void> {
    await apiDeleteFeedback(entry.id)
    putThread(entry.taskId, threadOf(entry.taskId).filter((candidate) => candidate.id !== entry.id))
  }

  /** The server puts the earlier description back, so the tasks are read again too. */
  async function discardRevision(entry: Feedback): Promise<void> {
    await apiDiscardRevision(entry.id)
    putThread(entry.taskId, threadOf(entry.taskId).filter((candidate) => candidate.id !== entry.id))
    await refreshes.refreshTasks()
  }

  return {
    feedbackByTask,
    selectedTaskFeedback,
    pendingCount,
    loadFeedback,
    addFeedback,
    editFeedback,
    deleteFeedback,
    discardRevision
  }
}
