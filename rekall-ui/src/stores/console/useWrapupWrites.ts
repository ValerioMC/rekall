import { restoreRevision as apiRestoreRevision } from '@/api/revisions.api'
import { deleteWrapup as apiDeleteWrapup, saveWrapup as apiSaveWrapup } from '@/api/wrapups.api'
import type { WrapupStreamEvent } from '@/model/catalog'
import type { TaskId } from '@/model/branded'
import type { RevisionKind } from '@/model/revision'
import type { ConsoleState } from './console-state'
import type { useConsoleRefreshes } from './useConsoleRefreshes'
import type { useTimers } from './useTimers'
import { trackSave, upsertById } from './rows'

/** A task's wrapup, written by hand or by a session. Tasks show whether they have one, so a first one or a deletion re-reads them. */
export function useWrapupWrites(
  state: ConsoleState,
  refreshes: ReturnType<typeof useConsoleRefreshes>,
  timers: ReturnType<typeof useTimers>
) {
  const { tasks, wrapups, paneFocus, saveState } = state

  async function saveWrapupBody(taskId: TaskId, bodyMarkdown: string): Promise<void> {
    const first = await trackSave(saveState, async () => {
      const saved = await apiSaveWrapup(taskId, bodyMarkdown)
      const known = wrapups.value.some((wrapup) => wrapup.id === saved.id)
      wrapups.value = upsertById(wrapups.value, saved, 'first')
      return !known
    })
    if (first) await refreshes.refreshTasks()
    await timers.resumeTimerOnWork(taskId)
  }

  async function removeWrapup(taskId: TaskId): Promise<void> {
    await apiDeleteWrapup(taskId)
    wrapups.value = wrapups.value.filter((wrapup) => wrapup.taskId !== taskId)
    paneFocus.value = 'note'
    await refreshes.refreshTasks()
  }

  function applyWrapupEvent(event: WrapupStreamEvent): void {
    if (event.deleted || !event.wrapup) {
      wrapups.value = wrapups.value.filter((wrapup) => wrapup.taskId !== event.taskId)
    } else {
      wrapups.value = upsertById(wrapups.value, event.wrapup, 'first')
    }
    tasks.value = tasks.value.map((task) =>
      task.id === event.taskId ? { ...task, hasWrapup: !event.deleted && event.wrapup !== null } : task
    )
  }

  /** Writes a kept revision back as the current text, and reloads whatever showed the old one. */
  async function restoreRevision(taskId: TaskId, revisionId: string): Promise<RevisionKind> {
    const restored = await apiRestoreRevision(taskId, revisionId)
    if (restored.kind === 'WRAPUP') await refreshes.refreshWrapups()
    await refreshes.refreshTasks()
    return restored.kind
  }

  return { saveWrapupBody, removeWrapup, applyWrapupEvent, restoreRevision }
}
