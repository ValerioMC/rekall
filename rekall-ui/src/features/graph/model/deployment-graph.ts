import type { ConnectionId, DeploymentGraphId, GraphStepId, ProjectId } from '@/model/branded'

/** The server's limits: a step decides between at most eight routes, a graph holds twenty steps. */
export const ROUTES_PER_STEP_MAX = 8
export const GRAPH_STEPS_MAX = 20

/** A route out of a graph's step: the outcome it leaves by, and the step it leads to or the end. */
export interface GraphRoute {
  readonly outcome: string
  readonly toStepId: GraphStepId | null
}

/** One step of a deployment graph: what a session does there and the connection it reaches. */
export interface GraphStep {
  readonly id: GraphStepId
  readonly title: string
  readonly detail: string | null
  readonly connectionId: ConnectionId | null
  readonly maxVisits: number | null
  readonly routes: readonly GraphRoute[]
}

/**
 * How a project releases its work: steps routed by outcome, each reaching one of the project's
 * connections, laid on a task as a flow when the task is attached to it.
 */
export interface DeploymentGraph {
  readonly id: DeploymentGraphId
  readonly projectId: ProjectId
  readonly title: string
  readonly description: string
  readonly steps: readonly GraphStep[]
  /** Why the graph cannot run unattended yet, so no task can be attached to it; null when it can. */
  readonly problem: string | null
  readonly createdAt: string
  readonly updatedAt: string
}

/** A route as the editor holds it: the key of the step it leads to, or null for the end. */
export interface DraftRoute {
  readonly outcome: string
  readonly toKey: string | null
}

/** A step as the editor holds it, under a key its routes point at; a saved step's key is its id. */
export interface DraftStep {
  readonly key: string
  readonly title: string
  readonly detail: string
  readonly connectionId: ConnectionId | null
  readonly maxVisits: number | null
  readonly routes: readonly DraftRoute[]
}

/** A graph as the editor holds it and the server takes it: always whole. */
export interface GraphDraft {
  readonly title: string
  readonly description: string
  readonly steps: readonly DraftStep[]
}
