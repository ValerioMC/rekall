import { computed, ref } from 'vue'
import {
  addFeedback as apiAddFeedback,
  deleteFeedback as apiDeleteFeedback,
  discardRevision as apiDiscardRevision,
  editFeedback as apiEditFeedback,
  fetchFeedback,
  startPass as apiStartPass
} from '@/features/feedback/api/feedback.api'
import { isRunning, isWaiting, type Feedback, type WrittenFeedbackKind } from '@/features/feedback/model/feedback'
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

  const waitingCount = computed<number>(() => selectedTaskFeedback.value.filter(isWaiting).length)

  const runningPass = computed<number | null>(
    () => selectedTaskFeedback.value.find(isRunning)?.pass ?? null
  )

  function putThread(taskId: TaskId, thread: readonly Feedback[]): void {
    feedbackByTask.value = { ...feedbackByTask.value, [taskId]: thread }
  }

  function threadOf(taskId: TaskId): readonly Feedback[] {
    return feedbackByTask.value[taskId] ?? []
  }

  async function loadFeedback(taskId: TaskId): Promise<void> {
    putThread(taskId, await fetchFeedback(taskId))
  }

  async function addFeedback(
    taskId: TaskId,
    kind: WrittenFeedbackKind,
    bodyMarkdown: string,
    followsSeq: number | null = null
  ): Promise<void> {
    const added = await trackSave(saveState, () => apiAddFeedback(taskId, kind, bodyMarkdown, followsSeq))
    putThread(taskId, upsertById(threadOf(taskId), added, 'last'))
  }

  /** Takes every waiting entry into the next pass; the entries it took replace their waiting rows. */
  async function startPass(taskId: TaskId): Promise<void> {
    const taken = await apiStartPass(taskId)
    putThread(taskId, taken.reduce((thread, entry) => replaceById(thread, entry), threadOf(taskId)))
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
    waitingCount,
    runningPass,
    loadFeedback,
    addFeedback,
    startPass,
    editFeedback,
    deleteFeedback,
    discardRevision
  }
}
