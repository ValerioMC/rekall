import {
  clearFlowPlaces,
  createStepLink,
  deleteStepLink,
  patchStepLink,
  placeStepInFlow,
  resetStepVisits,
  type FlowPoint,
  type StepLinkPatch
} from '@/api/step-flow.api'
import type { RouteTarget } from '@/common/flow/route-target'
import type { TaskStep } from '@/model/catalog'
import type { StepLinkId, TaskId, TaskStepId } from '@/model/branded'
import type { useStepWrites } from './useStepWrites'

/**
 * The routes between a task's steps, where steps were dragged on the canvas and the reset of the
 * visits. Each answer is the task's whole checklist, applied the way a stream event is.
 */
export function useStepFlowWrites(steps: ReturnType<typeof useStepWrites>) {
  function apply(taskId: TaskId, incoming: TaskStep[]): void {
    steps.applyStepEvent(taskId, incoming)
  }

  async function addRoute(taskId: TaskId, from: TaskStepId, to: RouteTarget, outcome: string): Promise<void> {
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

  async function placeStep(taskId: TaskId, id: TaskStepId, at: FlowPoint): Promise<void> {
    apply(taskId, await placeStepInFlow(id, at))
  }

  async function tidyFlow(taskId: TaskId): Promise<void> {
    apply(taskId, await clearFlowPlaces(taskId))
  }

  return { addRoute, editRoute, removeRoute, resetFlow, placeStep, tidyFlow }
}
