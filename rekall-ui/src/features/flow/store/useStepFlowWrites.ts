import {
  clearFlowPlaces,
  createStepLink,
  deleteStepLink,
  limitStepVisits,
  patchStepLink,
  placeStepInFlow,
  resetStepVisits,
  type FlowPoint,
  type StepLinkPatch
} from '@/features/flow/api/step-flow.api'
import type { RouteTarget } from '@/features/flow/lib/route-target'
import type { TaskStep } from '@/features/catalog/model/catalog'
import type { StepLinkId, TaskId, TaskStepId } from '@/model/branded'
import type { useStepWrites } from '../../steps/store/useStepWrites'

/**
 * The routes between a task's steps, where steps were dragged on the canvas, each step's visit
 * budget and the reset of the visits. Each answer is the task's whole checklist, applied the way a stream event is.
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

  async function limitVisits(taskId: TaskId, id: TaskStepId, maxVisits: number | null): Promise<void> {
    apply(taskId, await limitStepVisits(id, maxVisits))
  }

  async function tidyFlow(taskId: TaskId): Promise<void> {
    apply(taskId, await clearFlowPlaces(taskId))
  }

  return { addRoute, editRoute, removeRoute, resetFlow, placeStep, limitVisits, tidyFlow }
}
