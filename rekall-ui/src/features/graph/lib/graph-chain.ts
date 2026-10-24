import type { ConnectionKind } from '@/features/connection/model/connection'
import type { DraftStep } from '@/features/graph/model/deployment-graph'
import type { ConnectionId } from '@/model/branded'

/** One step of a graph as a chain shows it: its title and the kind of what it reaches. */
export interface ChainLink {
  readonly title: string
  readonly kind: ConnectionKind | null
}

/** The steps in order, each with the kind of its connection as `kindOf` knows it. */
export function chainOf(
  steps: readonly DraftStep[],
  kindOf: (id: ConnectionId) => ConnectionKind | null
): ChainLink[] {
  return steps.map((step) => ({ title: step.title, kind: step.connectionId ? kindOf(step.connectionId) : null }))
}

/** Whether a route leads back to a step at or before the one it leaves: the graph loops. */
export function loopsBack(steps: readonly DraftStep[]): boolean {
  const position = new Map(steps.map((step, at) => [step.key, at]))
  return steps.some((step, at) =>
    step.routes.some((route) => route.toKey !== null && (position.get(route.toKey) ?? Infinity) <= at)
  )
}
