import { computed } from 'vue'
import {
  deleteTimeEntry as apiDeleteTimeEntry,
  editTimeEntry as apiEditTimeEntry,
  startTimeEntry as apiStartTimeEntry,
  stopTimeEntry as apiStopTimeEntry
} from '@/features/time/api/time-entries.api'
import type { TimeEntryEdit } from '@/features/time/api/time-entries.api'
import type { TimeEntry } from '@/features/catalog/model/catalog'
import type { TaskId, TimeEntryId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'
import type { useConsoleRefreshes } from '../../../stores/console/useConsoleRefreshes'
import { upsertById } from '../../../stores/console/rows'

/** The time spent on each task: the timers a person runs by hand and the ones a console write resumes. */
export function useTimers(state: ConsoleState, refreshes: ReturnType<typeof useConsoleRefreshes>) {
  const { timeEntries, selectedTaskId } = state

  const runningEntries = computed(() => timeEntries.value.filter((entry) => entry.stoppedAt === null))

  const timerStartsInFlight = new Set<TaskId>()

  function upsertTimeEntry(entry: TimeEntry): void {
    timeEntries.value = upsertById(timeEntries.value, entry, 'first')
  }

  async function startTimer(taskId: TaskId): Promise<void> {
    upsertTimeEntry(await apiStartTimeEntry(taskId))
  }

  /**
   * Writing on a task is working on it, so a console write starts its paused timer. Only the
   * console calls this: a session's writes arrive over the feed and never start one.
   */
  async function resumeTimerOnWork(taskId: TaskId): Promise<void> {
    const running = runningEntries.value.some((entry) => entry.taskId === taskId)
    if (running || timerStartsInFlight.has(taskId)) return
    timerStartsInFlight.add(taskId)
    try {
      await startTimer(taskId)
    } finally {
      timerStartsInFlight.delete(taskId)
    }
  }

  /** A note can sit on several tasks: the one in view is being worked on, or else its only task. */
  async function resumeTimerOnNote(taskIds: readonly TaskId[]): Promise<void> {
    const worked =
      taskIds.find((id) => id === selectedTaskId.value) ?? (taskIds.length === 1 ? taskIds[0] : undefined)
    if (worked) await resumeTimerOnWork(worked)
  }

  /** Accepting a claim stops the task's timer on the server, so a running one is read again. */
  async function rereadTimerStoppedByAccept(taskId: TaskId): Promise<void> {
    if (runningEntries.value.some((entry) => entry.taskId === taskId)) await refreshes.refreshTimeEntries()
  }

  async function pauseTimer(taskId: TaskId): Promise<void> {
    upsertTimeEntry(await apiStopTimeEntry(taskId))
  }

  async function editTimer(id: TimeEntryId, input: TimeEntryEdit): Promise<void> {
    upsertTimeEntry(await apiEditTimeEntry(id, input))
  }

  async function deleteTimer(id: TimeEntryId): Promise<void> {
    await apiDeleteTimeEntry(id)
    timeEntries.value = timeEntries.value.filter((entry) => entry.id !== id)
  }

  return {
    runningEntries,
    startTimer,
    resumeTimerOnWork,
    resumeTimerOnNote,
    rereadTimerStoppedByAccept,
    pauseTimer,
    editTimer,
    deleteTimer
  }
}
