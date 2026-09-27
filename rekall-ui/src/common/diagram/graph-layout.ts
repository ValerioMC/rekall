import { graphlib, layout as runDagre } from '@dagrejs/dagre'
import { relationFamily, kindStyle, type NodeShape } from './kind-style'
import { textWidth, wrapText } from './text-wrap'
import type { Projection, ProjectedNode } from './graph-projection'
import type { Box, Point } from './viewport'

/**
 * Geometry for a projection, computed by dagre (layered layout: the flow reads in one direction,
 * crossings minimised). Deterministic for the same input, so a diagram never rearranges itself
 * between two visits.
 *
 * dagre cannot attach an edge to a compound node, so edges touching a cluster are left out of
 * its run and drawn afterwards as a straight segment between the two boxes' borders.
 */

export type Direction = 'LR' | 'TB'

export const NODE_WIDTH = 212
const ONE_LINE_HEIGHT = 56
const TWO_LINE_HEIGHT = 72
export const TITLE_FONT_SIZE = 12.5
/** Room a cluster keeps above its children for its own title. */
export const CLUSTER_HEADER = 30
const CLUSTER_PADDING = 16
const LABEL_FONT_SIZE = 11

/** Horizontal room each shape's outline takes from the title, beyond the glyph socket. */
const SHAPE_INSET: Readonly<Record<NodeShape, number>> = {
  soft: 0,
  rounded: 0,
  lozenge: 14,
  stadium: 10,
  signal: 16,
  cylinder: 0,
  subsystem: 10,
  block: 0
}

const GLYPH_COLUMN = 46
const RIGHT_PADDING = 30

export interface PlacedNode {
  readonly id: string
  readonly x: number
  readonly y: number
  readonly width: number
  readonly height: number
  readonly lines: readonly string[]
}

export interface PlacedCluster {
  readonly id: string
  readonly x: number
  readonly y: number
  readonly width: number
  readonly height: number
  readonly depth: number
}

export interface PlacedEdge {
  readonly key: string
  readonly points: readonly Point[]
  readonly labelAt: Point | null
}

export interface DiagramLayout {
  readonly nodes: ReadonlyMap<string, PlacedNode>
  readonly clusters: ReadonlyMap<string, PlacedCluster>
  readonly edges: readonly PlacedEdge[]
  readonly bounds: Box
}

/** Past this many nodes the second trial layout costs more than the better fit is worth. */
const AUTO_DIRECTION_LIMIT = 250
/** Roughly the canvas' shape between the library and the inspector. */
const NOMINAL_ASPECT = 1.15

/**
 * The direction whose layout fits a screen-shaped canvas at the larger scale: a long single
 * flow reads top to bottom, a wide fan left to right.
 */
export function preferredDirection(projection: Projection): Direction {
  if (projection.nodes.length > AUTO_DIRECTION_LIMIT) return 'LR'
  const fitScale = (direction: Direction): number => {
    const { width, height } = layoutProjection(projection, direction).bounds
    return Math.min(NOMINAL_ASPECT / Math.max(1, width), 1 / Math.max(1, height))
  }
  return fitScale('TB') > fitScale('LR') ? 'TB' : 'LR'
}

export function titleLines(node: ProjectedNode): string[] {
  const inset = SHAPE_INSET[kindStyle(node.node.kind).shape] * 2
  const room = NODE_WIDTH - GLYPH_COLUMN - RIGHT_PADDING - inset
  return wrapText(node.node.title, room, TITLE_FONT_SIZE, 2)
}

export function layoutProjection(projection: Projection, direction: Direction): DiagramLayout {
  const graph = new graphlib.Graph({ compound: true, multigraph: true })
  graph.setGraph({ rankdir: direction, nodesep: 36, ranksep: 72, edgesep: 18, marginx: 48, marginy: 48 })
  graph.setDefaultEdgeLabel(() => ({}))

  const lines = new Map<string, string[]>()
  const clusterIds = new Set(projection.nodes.filter((node) => node.isCluster).map((node) => node.id))
  for (const node of projection.nodes) {
    if (clusterIds.has(node.id)) {
      graph.setNode(node.id, { width: 0, height: 0 })
      continue
    }
    const wrapped = titleLines(node)
    lines.set(node.id, wrapped)
    graph.setNode(node.id, { width: NODE_WIDTH, height: wrapped.length > 1 ? TWO_LINE_HEIGHT : ONE_LINE_HEIGHT })
  }
  for (const node of projection.nodes) {
    if (node.parentId !== null && clusterIds.has(node.parentId)) graph.setParent(node.id, node.parentId)
  }

  const direct = new Set(projection.edges.filter((edge) => !clusterIds.has(edge.from) && !clusterIds.has(edge.to)))
  for (const edge of direct) {
    const family = relationFamily(edge.relation)
    const labelled = edge.label !== null && edge.label.length > 0
    graph.setEdge(
      edge.from,
      edge.to,
      {
        weight: family === 'flow' ? 3 : family === 'conditional' ? 2 : 1,
        minlen: 1,
        width: labelled ? textWidth(edge.label ?? '', LABEL_FONT_SIZE) + 18 : 0,
        height: labelled ? 20 : 0,
        labelpos: 'c'
      },
      edge.key
    )
  }

  runDagre(graph)

  const nodes = new Map<string, PlacedNode>()
  for (const node of projection.nodes) {
    if (clusterIds.has(node.id)) continue
    const placed = graph.node(node.id)
    const width = placed.width
    const height = placed.height
    nodes.set(node.id, { id: node.id, x: (placed.x ?? 0) - width / 2, y: (placed.y ?? 0) - height / 2, width, height, lines: lines.get(node.id) ?? [] })
  }

  const routed: PlacedEdge[] = [...direct].map((edge) => {
    const placed = graph.edge({ v: edge.from, w: edge.to, name: edge.key })
    const labelAt = edge.label && placed.x !== undefined && placed.y !== undefined ? { x: placed.x, y: placed.y } : null
    return { key: edge.key, points: placed.points ?? [], labelAt }
  })
  const opened = openFrameBands(projection, clusterIds, nodes, routed)

  const clusters = placeClusters(projection, clusterIds, opened.nodes)
  const boxOf = (id: string): Box | null => opened.nodes.get(id) ?? clusters.get(id) ?? null
  const routedByKey = new Map(opened.edges.map((edge) => [edge.key, edge]))

  const edges: PlacedEdge[] = []
  for (const edge of projection.edges) {
    const already = routedByKey.get(edge.key)
    if (already) {
      edges.push(already)
      continue
    }
    const from = boxOf(edge.from)
    const to = boxOf(edge.to)
    if (!from || !to) continue
    const start = borderPoint(from, centre(to))
    const end = borderPoint(to, centre(from))
    edges.push({ key: edge.key, points: [start, midpoint(start, end), end], labelAt: edge.label ? midpoint(start, end) : null })
  }

  return { nodes: opened.nodes, clusters, edges, bounds: boundsOf([...opened.nodes.values(), ...clusters.values()], edges) }
}

/**
 * Each frame is its members' union, grown by the padding and the title band, holding any nested
 * frame once that one has grown too; the deepest are settled first.
 */
function placeClusters(
  projection: Projection,
  clusterIds: ReadonlySet<string>,
  nodes: ReadonlyMap<string, PlacedNode>
): Map<string, PlacedCluster> {
  const parentOf = new Map(projection.nodes.map((node) => [node.id, node.parentId]))
  const depthOf = (id: string): number => {
    let depth = 0
    let parent = parentOf.get(id) ?? null
    while (parent !== null) {
      depth += 1
      parent = parentOf.get(parent) ?? null
    }
    return depth
  }
  const ordered = [...clusterIds].sort((a, b) => depthOf(b) - depthOf(a))
  const clusters = new Map<string, PlacedCluster>()
  for (const id of ordered) {
    const members: Box[] = projection.nodes
      .filter((node) => node.parentId === id)
      .map((node) => nodes.get(node.id) ?? clusters.get(node.id))
      .filter((box): box is PlacedNode | PlacedCluster => box !== undefined)
    const content = union(members)
    if (!content) continue
    clusters.set(id, {
      id,
      x: content.x - CLUSTER_PADDING,
      y: content.y - CLUSTER_PADDING - CLUSTER_HEADER,
      width: content.width + CLUSTER_PADDING * 2,
      height: content.height + CLUSTER_PADDING * 2 + CLUSTER_HEADER,
      depth: depthOf(id)
    })
  }
  return clusters
}

function union(boxes: readonly Box[]): Box | null {
  if (boxes.length === 0) return null
  const left = Math.min(...boxes.map((box) => box.x))
  const top = Math.min(...boxes.map((box) => box.y))
  const right = Math.max(...boxes.map((box) => box.x + box.width))
  const bottom = Math.max(...boxes.map((box) => box.y + box.height))
  return { x: left, y: top, width: right - left, height: bottom - top }
}

function boundsOf(boxes: readonly Box[], edges: readonly PlacedEdge[]): Box {
  const points = edges.flatMap((edge) => edge.points)
  const pointBoxes = points.map((point) => ({ x: point.x, y: point.y, width: 0, height: 0 }))
  return union([...boxes, ...pointBoxes]) ?? { x: 0, y: 0, width: 0, height: 0 }
}

function centre(box: Box): Point {
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 }
}

function midpoint(a: Point, b: Point): Point {
  return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }
}

/** Where the segment from the centre of `box` towards `toward` leaves the box. */
export function borderPoint(box: Box, toward: Point): Point {
  const origin = centre(box)
  const dx = toward.x - origin.x
  const dy = toward.y - origin.y
  if (dx === 0 && dy === 0) return origin
  const scale = Math.min(
    dx === 0 ? Number.POSITIVE_INFINITY : box.width / 2 / Math.abs(dx),
    dy === 0 ? Number.POSITIVE_INFINITY : box.height / 2 / Math.abs(dy)
  )
  return { x: origin.x + dx * scale, y: origin.y + dy * scale }
}

const BAND_GAP = 14

/**
 * dagre knows nothing of a frame's title band or padding, so a node outside a frame can sit
 * where the band goes. Where it would, a horizontal band is opened: everything from the frame's
 * content down moves by what is missing, and edges crossing the band stretch across it.
 */
function openFrameBands(
  projection: Projection,
  clusterIds: ReadonlySet<string>,
  placed: ReadonlyMap<string, PlacedNode>,
  routed: readonly PlacedEdge[]
): { nodes: Map<string, PlacedNode>; edges: PlacedEdge[] } {
  const nodes = new Map(placed)
  let edges = [...routed]
  const parentOf = new Map(projection.nodes.map((node) => [node.id, node.parentId]))
  const insideOf = (id: string, cluster: string): boolean => {
    let parent = parentOf.get(id) ?? null
    while (parent !== null) {
      if (parent === cluster) return true
      parent = parentOf.get(parent) ?? null
    }
    return false
  }

  const shiftFrom = (threshold: number, by: number): void => {
    for (const [id, node] of nodes) if (node.y >= threshold) nodes.set(id, { ...node, y: node.y + by })
    edges = edges.map((edge) => ({
      ...edge,
      points: edge.points.map((point) => (point.y >= threshold ? { x: point.x, y: point.y + by } : point)),
      labelAt: edge.labelAt && edge.labelAt.y >= threshold ? { x: edge.labelAt.x, y: edge.labelAt.y + by } : edge.labelAt
    }))
  }

  for (const cluster of clusterIds) {
    const members = () => [...nodes.values()].filter((node) => insideOf(node.id, cluster))
    const content = union(members())
    if (!content) continue
    const left = content.x - CLUSTER_PADDING
    const right = content.x + content.width + CLUSTER_PADDING
    const beside = [...nodes.values()].filter((node) => !insideOf(node.id, cluster) && node.x < right && node.x + node.width > left)

    const top = content.y - CLUSTER_PADDING - CLUSTER_HEADER
    const above = beside.filter((node) => node.y < content.y)
    const intrusion = Math.max(0, ...above.map((node) => node.y + node.height + BAND_GAP - top))
    if (intrusion > 0) shiftFrom(content.y - 0.5, intrusion)

    const grown = union(members())!
    const bottom = grown.y + grown.height + CLUSTER_PADDING
    const below = beside.map((node) => nodes.get(node.id)!).filter((node) => node.y + node.height > grown.y + grown.height)
    const overlap = Math.max(0, ...below.map((node) => bottom + BAND_GAP - node.y))
    if (overlap > 0) shiftFrom(grown.y + grown.height + 0.5, overlap)
  }
  return { nodes, edges }
}
