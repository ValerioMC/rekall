import {
  attachDeploymentGraph,
  releaseOnRun as apiReleaseOnRun,
  skipRelease as apiSkipRelease
} from '@/features/graph/api/deployment-graphs.api'
import { fetchTask } from '@/features/catalog/api/catalog.api'
import type { DeploymentGraphId, TaskId } from '@/model/branded'
import type { ConsoleState } from '../../../stores/console/console-state'
import { replaceById } from '../../../stores/console/rows'
import type { useStepWrites } from '../../steps/store/useStepWrites'

/**
 * Laying a project's deployment graph on a task, by hand or as the console runs it: the answer is
 * the task's new checklist, and the task is read again for when it was laid.
 */
export function useGraphAttach(state: ConsoleState, steps: ReturnType<typeof useStepWrites>) {
  async function refreshTask(taskId: TaskId): Promise<void> {
    state.tasks.value = replaceById(state.tasks.value, await fetchTask(taskId))
  }

  async function attachGraph(taskId: TaskId, graphId: DeploymentGraphId): Promise<void> {
    steps.applyStepEvent(taskId, await attachDeploymentGraph(taskId, graphId))
    await refreshTask(taskId)
  }

  /** The title of the release graph laid, or null when running the task laid nothing. */
  async function releaseOnRun(taskId: TaskId): Promise<string | null> {
    const answer = await apiReleaseOnRun(taskId)
    if (answer.laid === null) return null
    steps.applyStepEvent(taskId, answer.steps)
    await refreshTask(taskId)
    return answer.laid
  }

  async function skipRelease(taskId: TaskId, skipped: boolean): Promise<void> {
    state.tasks.value = replaceById(state.tasks.value, await apiSkipRelease(taskId, skipped))
  }

  return { attachGraph, releaseOnRun, skipRelease }
}
