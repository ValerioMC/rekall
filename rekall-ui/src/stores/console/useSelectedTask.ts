import { computed } from 'vue'
import type { CommitReference } from '@/features/commits/model/commitReference'
import { stepCompletedAt, stepIsComplete, type TaskReview } from '@/features/catalog/model/catalog'
import type { TaskStepId } from '@/model/branded'
import { reviewQueue as buildReviewQueue } from '@/features/catalog/model/review'
import type { ReviewItem } from '@/features/catalog/model/review'
import type { ConsoleState } from './console-state'

/** What the panes show about the task and note in view, and how far the task's wrapup lags behind its work. */
export function useSelectedTask(state: ConsoleState) {
  const { tasks, documents, wrapups, steps, timeEntries, commitReferences, selectedTaskId, selectedDocId } =
    state

  const selectedTask = computed(() => tasks.value.find((task) => task.id === selectedTaskId.value) ?? null)

  // One review queue, not two: a claimed step and a claimed stepless task are the same ask.
  const reviewQueue = computed<ReviewItem[]>(() => buildReviewQueue(tasks.value, steps.value))

  const selectedDocument = computed(
    () => documents.value.find((document) => document.id === selectedDocId.value) ?? null
  )

  const taskDocuments = computed(() =>
    selectedTaskId.value === null
      ? []
      : documents.value.filter((document) => document.tasks.some((ref) => ref.id === selectedTaskId.value))
  )

  const selectedTaskSteps = computed(() =>
    steps.value.filter((step) => step.taskId === selectedTaskId.value).sort((a, b) => a.position - b.position)
  )

  const selectedTaskCommitReferences = computed(() =>
    commitReferences.value
      .filter((reference) => reference.taskId === selectedTaskId.value)
      .sort((a, b) => b.createdAt.localeCompare(a.createdAt))
  )

  function stepCommitReferences(stepId: TaskStepId): CommitReference[] {
    return selectedTaskCommitReferences.value.filter((reference) => reference.stepId === stepId)
  }

  const openStepCount = computed(
    () => selectedTaskSteps.value.filter((step) => step.state !== 'DRAFT' && !step.done).length
  )

  const selectedTaskReview = computed<TaskReview | null>(() => {
    const task = selectedTask.value
    if (!task || !task.reviewActive) return null
    return {
      taskId: task.id,
      reviewState: task.reviewState,
      reviewActive: task.reviewActive,
      claimedAt: task.claimedAt,
      acceptedAt: task.acceptedAt,
      reviewNote: task.reviewNote
    }
  })

  const selectedWrapup = computed(
    () => wrapups.value.find((wrapup) => wrapup.taskId === selectedTaskId.value) ?? null
  )

  const wrapupMissesSteps = computed(() => {
    const wrapup = selectedWrapup.value
    if (!wrapup) return 0
    return selectedTaskSteps.value.filter((step) => {
      if (!stepIsComplete(step.state)) return false
      const at = stepCompletedAt(step)
      return at !== null && at > wrapup.updatedAt
    }).length
  })

  const runningStep = computed(() => selectedTaskSteps.value.find((step) => step.state === 'RUNNING') ?? null)

  const wrapupIsBehind = computed(() => {
    const wrapup = selectedWrapup.value
    if (!wrapup) return 0
    return taskDocuments.value.filter((document) => document.updatedAt > wrapup.updatedAt).length
  })

  const selectedTaskEntries = computed(() =>
    timeEntries.value
      .filter((entry) => entry.taskId === selectedTaskId.value)
      .sort((a, b) => b.startedAt.localeCompare(a.startedAt))
  )

  return {
    selectedTask,
    selectedDocument,
    reviewQueue,
    taskDocuments,
    selectedTaskSteps,
    selectedTaskCommitReferences,
    stepCommitReferences,
    openStepCount,
    selectedTaskReview,
    selectedWrapup,
    wrapupMissesSteps,
    runningStep,
    wrapupIsBehind,
    selectedTaskEntries
  }
}
