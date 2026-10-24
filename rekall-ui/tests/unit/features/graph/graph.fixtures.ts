import type { DeploymentGraph } from '@/features/graph/model/deployment-graph'
import type { DeploymentGraphId, GraphStepId, ProjectId } from '@/model/branded'

export const PROJECT = 'p-1' as ProjectId

/** Push then check, the check ending the flow or going back to the push. */
export function aGraph(overrides: Partial<DeploymentGraph> = {}): DeploymentGraph {
  return {
    id: 'g-1' as DeploymentGraphId,
    projectId: PROJECT,
    title: 'Release',
    description: 'How it ships.',
    problem: null,
    createdAt: '2026-10-24T10:00:00Z',
    updatedAt: '2026-10-24T10:00:00Z',
    steps: [
      { id: 's-1' as GraphStepId, title: 'Push commit', detail: null, connectionId: null, maxVisits: 3, flowX: null, flowY: null, routes: [{ outcome: 'pushed', toStepId: 's-2' as GraphStepId }] },
      {
        id: 's-2' as GraphStepId,
        title: 'Check',
        detail: null,
        connectionId: null,
        maxVisits: null, flowX: null, flowY: null,
        routes: [
          { outcome: 'healthy', toStepId: null },
          { outcome: 'error', toStepId: 's-1' as GraphStepId }
        ]
      }
    ],
    ...overrides
  }
}
