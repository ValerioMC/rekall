import type { StepLink, TaskStep } from '@/model/catalog'
import type { StepLinkId, TaskStepId } from '@/model/branded'

/** One move of the flow, in order: the step it entered and the route it left by. */
export interface FlowMove {
  readonly ordinal: number
  readonly step: TaskStep
  readonly outcome: string | null
  readonly open: boolean
}

/**
 * A checklist read as a flow: where the work stands, how it got there and which routes it has
 * walked. Derived from the steps alone, so a stream event redraws it with no request.
 */
export interface StepFlow {
  readonly isFlow: boolean
  readonly steps: readonly TaskStep[]
  readonly currentId: TaskStepId | null
  /** Where the flow begins: the first step on the checklist. */
  readonly entryId: TaskStepId | null
  readonly trail: readonly FlowMove[]
  /** Routes the work has left by at least once. */
  readonly walked: ReadonlySet<StepLinkId>
  /** The route the last move came in by, the one a live view animates. */
  readonly lastRouteId: StepLinkId | null
}

export function readFlow(all: readonly TaskStep[]): StepFlow {
  const steps = all.filter((step) => step.state !== 'DRAFT')
  const trail = movesOf(steps)
  const current = trail.find((move) => move.open)?.step ?? null
  return {
    isFlow: steps.some((step) => step.links.length > 0),
    steps,
    currentId: current?.id ?? null,
    entryId: steps[0]?.id ?? null,
    trail,
    walked: walkedRoutes(trail),
    lastRouteId: lastRoute(trail)
  }
}

export function routeTargets(
  step: TaskStep,
  steps: readonly TaskStep[]
): ReadonlyArray<{ link: StepLink; target: TaskStep | null }> {
  return step.links.map((link) => ({
    link,
    target: steps.find((candidate) => candidate.id === link.toStepId) ?? null
  }))
}

/** Routes arriving at `step`, with the step each one leaves from. */
export function routesInto(
  step: TaskStep,
  steps: readonly TaskStep[]
): ReadonlyArray<{ link: StepLink; source: TaskStep }> {
  return steps.flatMap((source) =>
    source.links.filter((link) => link.toStepId === step.id).map((link) => ({ link, source }))
  )
}

function movesOf(steps: readonly TaskStep[]): FlowMove[] {
  return steps
    .flatMap((step) =>
      step.visits.map((visit) => ({
        ordinal: visit.ordinal,
        step,
        outcome: visit.outcome,
        open: visit.leftAt === null
      }))
    )
    .sort((a, b) => a.ordinal - b.ordinal)
}

function routeTaken(move: FlowMove): StepLink | null {
  if (move.outcome === null) return null
  return move.step.links.find((link) => link.outcome === move.outcome) ?? null
}

function walkedRoutes(trail: readonly FlowMove[]): Set<StepLinkId> {
  const walked = new Set<StepLinkId>()
  for (const move of trail) {
    const link = routeTaken(move)
    if (link) walked.add(link.id)
  }
  return walked
}

/** The route into the step the flow stands on, when the move before it was left by one. */
function lastRoute(trail: readonly FlowMove[]): StepLinkId | null {
  const last = trail[trail.length - 1]
  const before = trail[trail.length - 2]
  if (!last?.open || !before) return null
  const link = routeTaken(before)
  return link?.toStepId === last.step.id ? link.id : null
}
