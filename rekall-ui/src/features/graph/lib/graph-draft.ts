import type {
  DeploymentGraph,
  DraftRoute,
  DraftStep,
  GraphDraft
} from '@/features/graph/model/deployment-graph'

export type Direction = 'up' | 'down'

/** The graph as the editor holds it: each step keyed by its id, each route by the id it leads to. */
export function draftOf(graph: DeploymentGraph): GraphDraft {
  return {
    title: graph.title,
    description: graph.description,
    steps: graph.steps.map((step) => ({
      key: step.id,
      title: step.title,
      detail: step.detail ?? '',
      connectionId: step.connectionId,
      maxVisits: step.maxVisits,
      flowX: step.flowX,
      flowY: step.flowY,
      routes: step.routes.map((route) => ({ outcome: route.outcome, toKey: route.toStepId }))
    }))
  }
}

/** Whether two drafts would save the same graph. */
export function sameDraft(a: GraphDraft, b: GraphDraft): boolean {
  return JSON.stringify(a) === JSON.stringify(b)
}

/**
 * Adds `step` after the step keyed `after`, or last. A step it follows that routes nowhere yet
 * gains a `done` route to it, so a graph built one step at a time reads as a line.
 */
export function insertStep(draft: GraphDraft, step: DraftStep, after: string | null): GraphDraft {
  const at = after === null ? -1 : draft.steps.findIndex((candidate) => candidate.key === after)
  const index = at === -1 ? draft.steps.length : at + 1
  const before = draft.steps[index - 1]
  const steps = draft.steps.map((candidate) =>
    candidate === before && candidate.routes.length === 0
      ? { ...candidate, routes: [{ outcome: 'done', toKey: step.key }] }
      : candidate
  )
  return { ...draft, steps: [...steps.slice(0, index), step, ...steps.slice(index)] }
}

export function updateStep(draft: GraphDraft, key: string, patch: Partial<Omit<DraftStep, 'key'>>): GraphDraft {
  return { ...draft, steps: draft.steps.map((step) => (step.key === key ? { ...step, ...patch } : step)) }
}

/** Removes the step and every route that led to it. */
export function removeStep(draft: GraphDraft, key: string): GraphDraft {
  return {
    ...draft,
    steps: draft.steps
      .filter((step) => step.key !== key)
      .map((step) => ({ ...step, routes: step.routes.filter((route) => route.toKey !== key) }))
  }
}

/** Moves the step one place; the first place is where the flow begins. */
export function moveStep(draft: GraphDraft, key: string, direction: Direction): GraphDraft {
  const from = draft.steps.findIndex((step) => step.key === key)
  const to = direction === 'up' ? from - 1 : from + 1
  if (from === -1 || to < 0 || to >= draft.steps.length) return draft
  const steps = [...draft.steps]
  const [moved] = steps.splice(from, 1)
  steps.splice(to, 0, moved!)
  return { ...draft, steps }
}

export function addRoute(draft: GraphDraft, key: string, route: DraftRoute): GraphDraft {
  return mapRoutes(draft, key, (routes) => [...routes, route])
}

export function updateRoute(draft: GraphDraft, key: string, index: number, patch: Partial<DraftRoute>): GraphDraft {
  return mapRoutes(draft, key, (routes) => routes.map((route, at) => (at === index ? { ...route, ...patch } : route)))
}

export function removeRoute(draft: GraphDraft, key: string, index: number): GraphDraft {
  return mapRoutes(draft, key, (routes) => routes.filter((_, at) => at !== index))
}

/**
 * An outcome for a new route out of `step` that it does not use yet: `done` toward the end or
 * `next` toward a step, numbered when taken.
 */
export function freeOutcome(step: DraftStep, toEnd: boolean): string {
  const base = toEnd ? 'done' : 'next'
  const taken = new Set(step.routes.map((route) => route.outcome))
  if (!taken.has(base)) return base
  let number = 2
  while (taken.has(`${base}-${number}`)) number += 1
  return `${base}-${number}`
}

function mapRoutes(
  draft: GraphDraft,
  key: string,
  change: (routes: readonly DraftRoute[]) => readonly DraftRoute[]
): GraphDraft {
  return updateStep(draft, key, { routes: change(draft.steps.find((step) => step.key === key)?.routes ?? []) })
}
