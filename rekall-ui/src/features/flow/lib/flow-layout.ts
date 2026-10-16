import { graphlib, layout as runDagre } from '@dagrejs/dagre'
import { textWidth } from '@/features/diagram/lib/text-wrap'
import type { Box, Point } from '@/features/diagram/lib/viewport'
import type { TaskStep } from '@/features/catalog/model/catalog'
import type { StepLinkId, TaskStepId } from '@/model/branded'

export const NODE_WIDTH = 232
export const NODE_HEIGHT = 76
export const TERMINAL_WIDTH = 128
export const TERMINAL_HEIGHT = 40
const LABEL_FONT_SIZE = 11
const MARGIN = 32
/** How far a loop leaves the side of a dragged step before it turns back. */
const LOOP_REACH = 44

const START = 'flow:start'
const END = 'flow:end'

export type PlacedRouteKind = 'entry' | 'route' | 'exit'

export interface PlacedRoute {
  /** The route's id for a real one, `entry` for Start's, `exit:<step>` for a step that ends the flow unrouted. */
  readonly key: string
  readonly kind: PlacedRouteKind
  readonly linkId: StepLinkId | null
  readonly from: TaskStepId | typeof START
  readonly to: TaskStepId | typeof END
  readonly points: readonly Point[]
  readonly labelAt: Point
}

export interface FlowLayout {
  readonly nodes: ReadonlyMap<TaskStepId, Box>
  readonly start: Box
  readonly end: Box
  readonly routes: readonly PlacedRoute[]
  readonly width: number
  readonly height: number
}

type Endpoint = TaskStepId | typeof START | typeof END

/**
 * Where each step, Start, End and route sits, top to bottom. dagre breaks the loops itself and is
 * deterministic, so the drawing holds still while the work moves. Start leads to the first step;
 * End collects the routes that finish the flow and, on a flow, the steps that have none.
 */
export function layoutFlow(steps: readonly TaskStep[]): FlowLayout {
  const graph = new graphlib.Graph({ multigraph: true })
  graph.setGraph({ rankdir: 'TB', nodesep: 56, ranksep: 72, edgesep: 24, marginx: MARGIN, marginy: MARGIN })
  graph.setDefaultEdgeLabel(() => ({}))
  const ids = new Set(steps.map((step) => step.id))
  const isFlow = steps.some((step) => step.links.length > 0)
  graph.setNode(START, { width: TERMINAL_WIDTH, height: TERMINAL_HEIGHT })
  graph.setNode(END, { width: TERMINAL_WIDTH, height: TERMINAL_HEIGHT })
  for (const step of steps) graph.setNode(step.id, { width: NODE_WIDTH, height: NODE_HEIGHT })

  const drawn: Array<Omit<PlacedRoute, 'points' | 'labelAt'> & { edge: graphlib.Edge }> = []
  const connect = (
    from: Endpoint,
    to: Endpoint,
    key: string,
    kind: PlacedRouteKind,
    linkId: StepLinkId | null,
    label = { width: 0, height: 0 }
  ): void => {
    graph.setEdge(from, to, { minlen: 1, ...label, labelpos: 'c' }, key)
    drawn.push({
      key,
      kind,
      linkId,
      from: from as PlacedRoute['from'],
      to: to as PlacedRoute['to'],
      edge: { v: from, w: to, name: key }
    })
  }

  const entry = steps[0]
  if (entry) connect(START, entry.id, 'entry', 'entry', null)
  chainInChecklistOrder(graph, steps)
  for (const step of steps) {
    for (const link of step.links) {
      const to = link.toStepId ?? END
      if (to !== END && !ids.has(to)) continue
      const label = { width: textWidth(link.outcome, LABEL_FONT_SIZE) + 22, height: 22 }
      connect(step.id, to, link.id, 'route', link.id, label)
    }
    if (isFlow && step.links.length === 0) connect(step.id, END, `exit:${step.id}`, 'exit', null)
  }
  runDagre(graph)

  const nodes = new Map<TaskStepId, Box>()
  for (const step of steps) nodes.set(step.id, boxAt(graph.node(step.id), NODE_WIDTH, NODE_HEIGHT))
  const routes: PlacedRoute[] = drawn.map(({ edge, ...route }) => {
    const at = graph.edge(edge)
    const points = at.points ?? []
    return { ...route, points, labelAt: { x: at.x ?? points[0]?.x ?? 0, y: at.y ?? points[0]?.y ?? 0 } }
  })
  const size = graph.graph()
  return {
    nodes,
    start: boxAt(graph.node(START), TERMINAL_WIDTH, TERMINAL_HEIGHT),
    end: boxAt(graph.node(END), TERMINAL_WIDTH, TERMINAL_HEIGHT),
    routes,
    width: size.width ?? 0,
    height: size.height ?? 0
  }
}

/**
 * The layout with some steps where the person put them. A route touching a moved step is drawn
 * between the boxes themselves, since dagre routed it around the place the step used to be.
 */
export function placeSteps(layout: FlowLayout, places: ReadonlyMap<TaskStepId, Point>): FlowLayout {
  if (places.size === 0) return layout
  const nodes = new Map<TaskStepId, Box>()
  for (const [id, box] of layout.nodes) {
    const at = places.get(id)
    nodes.set(id, at ? { ...box, x: at.x, y: at.y } : box)
  }
  const boxOf = (endpoint: Endpoint): Box =>
    endpoint === START ? layout.start : endpoint === END ? layout.end : nodes.get(endpoint)!
  const routes = layout.routes.map((route) => {
    const moved = places.has(route.from as TaskStepId) || places.has(route.to as TaskStepId)
    if (!moved) return route
    const points = routeBetween(boxOf(route.from), boxOf(route.to))
    return { ...route, points, labelAt: midpoint(points) }
  })
  const boxes = [...nodes.values(), layout.start, layout.end]
  return {
    ...layout,
    nodes,
    routes,
    width: Math.max(layout.width, ...boxes.map((box) => box.x + box.width + MARGIN)),
    height: Math.max(layout.height, ...boxes.map((box) => box.y + box.height + MARGIN))
  }
}

/** A route between two boxes: down when the target lies below, out of the right sides when it does not. */
export function routeBetween(from: Box, to: Box): Point[] {
  if (to.y >= from.y + from.height) {
    const start = { x: from.x + from.width / 2, y: from.y + from.height }
    const end = { x: to.x + to.width / 2, y: to.y }
    const middle = (start.y + end.y) / 2
    return [start, { x: start.x, y: middle }, { x: end.x, y: middle }, end]
  }
  const start = { x: from.x + from.width, y: from.y + from.height / 2 }
  const end = { x: to.x + to.width, y: to.y + to.height / 2 }
  const reach = Math.max(start.x, end.x) + LOOP_REACH
  return [start, { x: reach, y: start.y }, { x: reach, y: end.y }, end]
}

function midpoint(points: readonly Point[]): Point {
  const inner = points.slice(1, -1)
  const pair = inner.length >= 2 ? inner : points
  const first = pair[0]!
  const last = pair[pair.length - 1]!
  return { x: (first.x + last.x) / 2, y: (first.y + last.y) / 2 }
}

function boxAt(at: { x?: number; y?: number }, width: number, height: number): Box {
  return { x: (at.x ?? 0) - width / 2, y: (at.y ?? 0) - height / 2, width, height }
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
  const last = steps[steps.length - 1]
  if (last) graph.setEdge(last.id, END, { minlen: 1, weight: 0.2, width: 0, height: 0 }, 'chain:end')
}

export const FLOW_START_KEY = START
export const FLOW_END_KEY = END
