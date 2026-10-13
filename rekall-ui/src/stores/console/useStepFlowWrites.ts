import {
  createStepLink,
  deleteStepLink,
  patchStepLink,
  resetStepVisits,
  type StepLinkPatch
} from '@/api/step-flow.api'
import type { TaskStep } from '@/model/catalog'
import type { StepLinkId, TaskId, TaskStepId } from '@/model/branded'
import type { useStepWrites } from './useStepWrites'

/**
 * The routes between a task's steps and the reset of its visits. Each answer is the task's whole
 * checklist, applied the way a stream event is.
 */
export function useStepFlowWrites(steps: ReturnType<typeof useStepWrites>) {
  function apply(taskId: TaskId, incoming: TaskStep[]): void {
    steps.applyStepEvent(taskId, incoming)
  }

  async function addRoute(taskId: TaskId, from: TaskStepId, to: TaskStepId, outcome: string): Promise<void> {
    apply(taskId, await createStepLink(from, to, outcome))
  }

  async function editRoute(taskId: TaskId, id: StepLinkId, patch: StepLinkPatch): Promise<void> {
    apply(taskId, await patchStepLink(id, patch))
  }

  async function removeRoute(taskId: TaskId, id: StepLinkId): Promise<void> {
    apply(taskId, await deleteStepLink(id))
  }

  async function resetFlow(taskId: TaskId): Promise<void> {
    apply(taskId, await resetStepVisits(taskId))
  }

  return { addRoute, editRoute, removeRoute, resetFlow }
}
