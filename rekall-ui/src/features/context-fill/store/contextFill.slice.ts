import { computed, ref } from 'vue'
import { fetchContextFills } from '@/features/context-fill/api/context-fill.api'
import { fillOf } from '@/features/context-fill/lib/context-fill-level'
import type { ContextFill, ContextFillStreamEvent, ContextFillSubject } from '@/features/context-fill/model/context-fill'
import type { TaskId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'

const NO_FILLS: readonly ContextFill[] = []

/**
 * The last context fill of each task and of each of its steps, read when the task is shown and
 * replaced whole when the server says a turn changed one.
 */
export function useContextFills(state: ConsoleState) {
  const { selectedTaskId } = state
  const contextFillsByTask = ref<Readonly<Record<string, readonly ContextFill[]>>>({})

  const selectedTaskContextFills = computed<readonly ContextFill[]>(() => fillsOf(selectedTaskId.value))

  function fillsOf(taskId: TaskId | null): readonly ContextFill[] {
    return taskId === null ? NO_FILLS : (contextFillsByTask.value[taskId] ?? NO_FILLS)
  }

  function putContextFills(taskId: TaskId, fills: readonly ContextFill[]): void {
    contextFillsByTask.value = { ...contextFillsByTask.value, [taskId]: fills }
  }

  async function loadContextFills(taskId: TaskId): Promise<void> {
    putContextFills(taskId, await fetchContextFills(taskId))
  }

  function applyContextFillEvent(event: ContextFillStreamEvent): void {
    putContextFills(event.taskId, event.fills)
  }

  function contextFillOf(subject: ContextFillSubject): ContextFill | null {
    return fillOf(fillsOf(subject.taskId), subject.stepId)
  }

  return { contextFillsByTask, selectedTaskContextFills, loadContextFills, applyContextFillEvent, contextFillOf }
}
