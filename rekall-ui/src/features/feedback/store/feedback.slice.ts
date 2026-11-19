import { computed, ref } from 'vue'
import {
  discardRevision as apiDiscardRevision,
  fetchFeedback,
  retargetFeedback as apiRetargetFeedback,
  startPass as apiStartPass
} from '@/features/feedback/api/feedback.api'
import { isRunning, isWaiting, type Feedback, type FeedbackKind } from '@/features/feedback/model/feedback'
import type { TaskId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'
import type { useConsoleRefreshes } from '../../../stores/console/useConsoleRefreshes'
import { replaceById, trackSave } from '../../../stores/console/rows'

/** The revisions of each task's description: read when the task is shown and whenever the server says it changed. */
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

  /** Takes every waiting entry into the next pass; the entries it took replace their waiting rows. */
  async function startPass(taskId: TaskId): Promise<void> {
    const taken = await apiStartPass(taskId)
    putThread(taskId, taken.reduce((thread, entry) => replaceById(thread, entry), threadOf(taskId)))
  }

  async function retargetFeedback(entry: Feedback, kind: FeedbackKind): Promise<void> {
    const retargeted = await trackSave(saveState, () => apiRetargetFeedback(entry.id, kind))
    putThread(entry.taskId, replaceById(threadOf(entry.taskId), retargeted))
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
    startPass,
    retargetFeedback,
    discardRevision
  }
}
