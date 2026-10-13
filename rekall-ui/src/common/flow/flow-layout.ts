import { graphlib, layout as runDagre } from '@dagrejs/dagre'
import { textWidth } from '@/common/diagram/text-wrap'
import type { Box, Point } from '@/common/diagram/viewport'
import type { TaskStep } from '@/model/catalog'
import type { StepLinkId, TaskStepId } from '@/model/branded'

export const NODE_WIDTH = 232
export const NODE_HEIGHT = 76
const LABEL_FONT_SIZE = 11
const MARGIN = 32

export interface PlacedRoute {
  readonly id: StepLinkId
  readonly points: readonly Point[]
  readonly labelAt: Point
}

export interface FlowLayout {
  readonly nodes: ReadonlyMap<TaskStepId, Box>
  readonly routes: readonly PlacedRoute[]
  readonly width: number
  readonly height: number
}

/**
 * Where each step and route sits, top to bottom. dagre breaks the loops itself and is
 * deterministic, so the drawing holds still while the work moves.
 */
export function layoutFlow(steps: readonly TaskStep[]): FlowLayout {
  const graph = new graphlib.Graph({ multigraph: true })
  graph.setGraph({ rankdir: 'TB', nodesep: 56, ranksep: 72, edgesep: 24, marginx: MARGIN, marginy: MARGIN })
  graph.setDefaultEdgeLabel(() => ({}))
  const ids = new Set(steps.map((step) => step.id))
  for (const step of steps) graph.setNode(step.id, { width: NODE_WIDTH, height: NODE_HEIGHT })
  chainInChecklistOrder(graph, steps)
  for (const step of steps) {
    for (const link of step.links) {
      if (!ids.has(link.toStepId)) continue
      const label = { width: textWidth(link.outcome, LABEL_FONT_SIZE) + 22, height: 22, labelpos: 'c' }
      graph.setEdge(step.id, link.toStepId, { minlen: 1, ...label }, link.id)
    }
  }
  runDagre(graph)

  const nodes = new Map<TaskStepId, Box>()
  for (const step of steps) {
    const at = graph.node(step.id)
    nodes.set(step.id, {
      x: (at.x ?? 0) - NODE_WIDTH / 2,
      y: (at.y ?? 0) - NODE_HEIGHT / 2,
      width: NODE_WIDTH,
      height: NODE_HEIGHT
    })
  }
  const routes: PlacedRoute[] = steps.flatMap((step) =>
    step.links
      .filter((link) => ids.has(link.toStepId))
      .map((link) => {
        const at = graph.edge({ v: step.id, w: link.toStepId, name: link.id })
        const points = at.points ?? []
        return {
          id: link.id,
          points,
          labelAt: { x: at.x ?? points[0]?.x ?? 0, y: at.y ?? points[0]?.y ?? 0 }
        }
      })
  )
  const size = graph.graph()
  return { nodes, routes, width: size.width ?? 0, height: size.height ?? 0 }
}

/**
 * Unseen edges between consecutive steps keep the spine in checklist order, so steps with no
 * routes yet stack like the list instead of lining up.
 */
function chainInChecklistOrder(graph: graphlib.Graph, steps: readonly TaskStep[]): void {
  steps.slice(1).forEach((step, index) => {
    graph.setEdge(
      steps[index]!.id,
      step.id,
      { minlen: 1, weight: 0.2, width: 0, height: 0 },
      `chain:${index}`
    )
  })
}
