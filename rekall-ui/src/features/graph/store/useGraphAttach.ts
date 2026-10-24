import { attachDeploymentGraph } from '@/features/graph/api/deployment-graphs.api'
import type { DeploymentGraphId, TaskId } from '@/model/branded'
import type { useStepWrites } from '../../steps/store/useStepWrites'

/** Laying a project's deployment graph on a task: its answer is the task's new checklist. */
export function useGraphAttach(steps: ReturnType<typeof useStepWrites>) {
  async function attachGraph(taskId: TaskId, graphId: DeploymentGraphId): Promise<void> {
    steps.applyStepEvent(taskId, await attachDeploymentGraph(taskId, graphId))
  }

  return { attachGraph }
}
