import { computed, ref } from 'vue'
import {
  addDecision as apiAddDecision,
  editDecision as apiEditDecision,
  fetchDecisions,
  removeDecision as apiRemoveDecision
} from '@/features/decisions/api/decisions.api'
import { NO_DECISIONS, type Decision, type DecisionEdit, type NewDecision, type TaskDecisions } from '@/features/decisions/model/decision'
import type { TaskId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'
import { replaceById, trackSave } from '../../../stores/console/rows'

/**
 * The decisions of each task, read when the task is shown and whenever the server says they changed.
 * They run parallel to the plan: they add no count anywhere. The text a session reads comes from the
 * server with the decisions, never built here.
 */
export function useTaskDecisions(state: ConsoleState) {
  const { selectedTaskId, saveState } = state
  const decisionsByTask = ref<Readonly<Record<string, TaskDecisions>>>({})

  const selectedTaskDecisions = computed<readonly Decision[]>(() => decisionsOf(selectedTaskId.value).decisions)

  const selectedTaskSessionBlock = computed<string>(() => decisionsOf(selectedTaskId.value).sessionBlock)

  function decisionsOf(taskId: TaskId | null): TaskDecisions {
    return taskId === null ? NO_DECISIONS : (decisionsByTask.value[taskId] ?? NO_DECISIONS)
  }

  function putDecisions(taskId: TaskId, decisions: TaskDecisions): void {
    decisionsByTask.value = { ...decisionsByTask.value, [taskId]: decisions }
  }

  async function loadDecisions(taskId: TaskId): Promise<void> {
    putDecisions(taskId, await fetchDecisions(taskId))
  }

  async function addDecision(taskId: TaskId, decision: NewDecision): Promise<Decision> {
    const added = await trackSave(saveState, () => apiAddDecision(taskId, decision))
    await loadDecisions(taskId)
    return added
  }

  /** The row changes in place; the session block is read again, since the server renders it. */
  async function editDecision(decision: Decision, edit: DecisionEdit): Promise<Decision> {
    const edited = await trackSave(saveState, () => apiEditDecision(decision.taskId, decision.id, edit))
    const current = decisionsOf(decision.taskId)
    putDecisions(decision.taskId, { ...current, decisions: replaceById(current.decisions, edited) })
    await loadDecisions(decision.taskId)
    return edited
  }

  async function removeDecision(decision: Decision): Promise<void> {
    await trackSave(saveState, () => apiRemoveDecision(decision.id))
    const current = decisionsOf(decision.taskId)
    putDecisions(decision.taskId, { ...current, decisions: current.decisions.filter((candidate) => candidate.id !== decision.id) })
    await loadDecisions(decision.taskId)
  }

  return {
    decisionsByTask,
    selectedTaskDecisions,
    selectedTaskSessionBlock,
    loadDecisions,
    addDecision,
    editDecision,
    removeDecision
  }
}
