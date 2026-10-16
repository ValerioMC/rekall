import { graphlib, layout as runDagre } from '@dagrejs/dagre'
import { textWidth } from '../text-wrap'
import { borderPoint, centre, midpoint, pointAlong, union } from './geometry'
import { sizeOf } from './node-size'
import type { GraphScene } from '../scene/graph-scene'
import type { Box, Point } from '../viewport'

/**
 * Geometry for a graph sheet, computed by dagre in the direction the sheet declares.
 * Deterministic for the same sheet, so a diagram never rearranges itself between two visits.
 *
 * dagre cannot attach an edge to a group, so an edge touching one is ranked through a member
 * standing in for the group, then drawn as a straight segment between the two boxes' borders.
 */

/** Room a group keeps above its members for its own title. */
export const GROUP_HEADER = 30
const GROUP_PADDING = 16
const LABEL_FONT_SIZE = 11
const END_LABEL_OFFSET = 22
const BAND_GAP = 14

export interface PlacedNode extends Box {
  readonly id: string
  readonly lines: readonly string[]
}

export interface PlacedGroup extends Box {
  readonly id: string
  readonly depth: number
}

export interface PlacedEdge {
  readonly key: string
  readonly points: readonly Point[]
  readonly labelAt: Point | null
  readonly startLabelAt: Point | null
  readonly endLabelAt: Point | null
}

export interface GraphLayout {
  readonly nodes: ReadonlyMap<string, PlacedNode>
  readonly groups: ReadonlyMap<string, PlacedGroup>
  readonly edges: readonly PlacedEdge[]
  readonly bounds: Box
}

export function layoutGraph(scene: GraphScene): GraphLayout {
  const graph = new graphlib.Graph({ compound: true, multigraph: true })
  graph.setGraph({ rankdir: scene.direction, nodesep: 40, ranksep: 64, edgesep: 18, marginx: 48, marginy: 48 })
  graph.setDefaultEdgeLabel(() => ({}))

  const groupIds = new Set(scene.groups.map((group) => group.id))
  const lines = new Map<string, readonly string[]>()
  for (const group of scene.groups) graph.setNode(group.id, { width: 0, height: 0 })
  for (const node of scene.nodes) {
    const size = sizeOf(node, scene.direction)
    lines.set(node.id, size.lines)
    graph.setNode(node.id, { width: size.width, height: size.height })
  }
  for (const member of [...scene.nodes, ...scene.groups]) {
    if (member.parentId !== null && groupIds.has(member.parentId)) graph.setParent(member.id, member.parentId)
  }

  const direct = scene.edges.filter((edge) => !groupIds.has(edge.from) && !groupIds.has(edge.to))
  const standIn = groupStandIns(scene)
  for (const edge of scene.edges.filter((candidate) => !direct.includes(candidate))) {
    const from = groupIds.has(edge.from) ? standIn.exit.get(edge.from) : edge.from
    const to = groupIds.has(edge.to) ? standIn.entry.get(edge.to) : edge.to
    if (from && to && from !== to) graph.setEdge(from, to, { minlen: 1, weight: 1 }, edge.key)
  }
  for (const edge of direct) {
    const labelled = edge.label !== null
    graph.setEdge(
      edge.from,
      edge.to,
      { minlen: 1, width: labelled ? textWidth(edge.label ?? '', LABEL_FONT_SIZE) + 18 : 0, height: labelled ? 20 : 0, labelpos: 'c' },
      edge.key
    )
  }

  runDagre(graph)

  const placed = new Map<string, PlacedNode>()
  for (const node of scene.nodes) {
    const at = graph.node(node.id)
    placed.set(node.id, { id: node.id, x: (at.x ?? 0) - at.width / 2, y: (at.y ?? 0) - at.height / 2, width: at.width, height: at.height, lines: lines.get(node.id) ?? [] })
  }
  const routed = direct.map((edge) => {
    const at = graph.edge({ v: edge.from, w: edge.to, name: edge.key })
    const labelAt = edge.label !== null && at.x !== undefined && at.y !== undefined ? { x: at.x, y: at.y } : null
    return withEndLabels(edge.key, at.points ?? [], labelAt)
  })

  const opened = openGroupBands(scene, groupIds, placed, routed)
  const groups = placeGroups(scene, opened.nodes)
  const boxOf = (id: string): Box | null => opened.nodes.get(id) ?? groups.get(id) ?? null
  const routedByKey = new Map(opened.edges.map((edge) => [edge.key, edge]))

  const edges: PlacedEdge[] = []
  for (const edge of scene.edges) {
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
    edges.push(withEndLabels(edge.key, [start, midpoint(start, end), end], edge.label !== null ? midpoint(start, end) : null))
  }
  return { nodes: opened.nodes, groups, edges, bounds: boundsOf([...opened.nodes.values(), ...groups.values()], edges) }
}

/**
 * Which member stands for a group in the ranking: the first node inside it for what comes in,
 * the last for what goes out, so the group lands between them in the flow.
 */
function groupStandIns(scene: GraphScene): { entry: Map<string, string>; exit: Map<string, string> } {
  const parentOf = new Map<string, string | null>([...scene.nodes, ...scene.groups].map((member) => [member.id, member.parentId]))
  const inside = (id: string, group: string): boolean => {
    let parent = parentOf.get(id) ?? null
    while (parent !== null) {
      if (parent === group) return true
      parent = parentOf.get(parent) ?? null
    }
    return false
  }
  const entry = new Map<string, string>()
  const exit = new Map<string, string>()
  for (const group of scene.groups) {
    const members = scene.nodes.filter((node) => inside(node.id, group.id))
    const first = members[0]
    const last = members[members.length - 1]
    if (first) entry.set(group.id, first.id)
    if (last) exit.set(group.id, last.id)
  }
  return { entry, exit }
}

function withEndLabels(key: string, points: readonly Point[], labelAt: Point | null): PlacedEdge {
  return {
    key,
    points,
    labelAt,
    startLabelAt: pointAlong(points, END_LABEL_OFFSET),
    endLabelAt: pointAlong([...points].reverse(), END_LABEL_OFFSET)
  }
}

function depthsOf(scene: GraphScene): (id: string) => number {
  const parentOf = new Map<string, string | null>([...scene.nodes, ...scene.groups].map((member) => [member.id, member.parentId]))
  return (id) => {
    let depth = 0
    let parent = parentOf.get(id) ?? null
    while (parent !== null) {
      depth += 1
      parent = parentOf.get(parent) ?? null
    }
    return depth
  }
}

/**
 * Each group is its members' union, grown by the padding and the title band, holding any nested
 * group once that one has grown too; the deepest are settled first.
 */
function placeGroups(scene: GraphScene, nodes: ReadonlyMap<string, PlacedNode>): Map<string, PlacedGroup> {
  const depthOf = depthsOf(scene)
  const ordered = [...scene.groups].sort((a, b) => depthOf(b.id) - depthOf(a.id))
  const groups = new Map<string, PlacedGroup>()
  for (const group of ordered) {
    const members: Box[] = [...scene.nodes, ...scene.groups]
      .filter((member) => member.parentId === group.id)
      .map((member) => nodes.get(member.id) ?? groups.get(member.id))
      .filter((box): box is PlacedNode | PlacedGroup => box !== undefined)
    const content = union(members)
    if (!content) continue
    groups.set(group.id, {
      id: group.id,
      x: content.x - GROUP_PADDING,
      y: content.y - GROUP_PADDING - GROUP_HEADER,
      width: content.width + GROUP_PADDING * 2,
      height: content.height + GROUP_PADDING * 2 + GROUP_HEADER,
      depth: depthOf(group.id)
    })
  }
  return groups
}

/**
 * dagre knows nothing of a group's title band or padding, so a node outside a group can sit
 * where the band goes. Where it would, a horizontal band is opened: everything from the group's
 * content down moves by what is missing, and edges crossing the band stretch across it.
 */
function openGroupBands(
  scene: GraphScene,
  groupIds: ReadonlySet<string>,
  placed: ReadonlyMap<string, PlacedNode>,
  routed: readonly PlacedEdge[]
): { nodes: Map<string, PlacedNode>; edges: PlacedEdge[] } {
  const nodes = new Map(placed)
  let edges = [...routed]
  const parentOf = new Map<string, string | null>([...scene.nodes, ...scene.groups].map((member) => [member.id, member.parentId]))
  const insideOf = (id: string, group: string): boolean => {
    let parent = parentOf.get(id) ?? null
    while (parent !== null) {
      if (parent === group) return true
      parent = parentOf.get(parent) ?? null
    }
    return false
  }
  const moved = (point: Point | null, threshold: number, by: number): Point | null =>
    point && point.y >= threshold ? { x: point.x, y: point.y + by } : point

  const shiftFrom = (threshold: number, by: number): void => {
    for (const [id, node] of nodes) if (node.y >= threshold) nodes.set(id, { ...node, y: node.y + by })
    edges = edges.map((edge) => ({
      ...edge,
      points: edge.points.map((point) => moved(point, threshold, by)!),
      labelAt: moved(edge.labelAt, threshold, by),
      startLabelAt: moved(edge.startLabelAt, threshold, by),
      endLabelAt: moved(edge.endLabelAt, threshold, by)
    }))
  }

  for (const group of groupIds) {
    const members = (): PlacedNode[] => [...nodes.values()].filter((node) => insideOf(node.id, group))
    const content = union(members())
    if (!content) continue
    const left = content.x - GROUP_PADDING
    const right = content.x + content.width + GROUP_PADDING
    const beside = [...nodes.values()].filter((node) => !insideOf(node.id, group) && node.x < right && node.x + node.width > left)

    const top = content.y - GROUP_PADDING - GROUP_HEADER
    const above = beside.filter((node) => node.y < content.y)
    const intrusion = Math.max(0, ...above.map((node) => node.y + node.height + BAND_GAP - top))
    if (intrusion > 0) shiftFrom(content.y - 0.5, intrusion)

    const grown = union(members())
    if (!grown) continue
    const bottom = grown.y + grown.height + GROUP_PADDING
    const below = beside.map((node) => nodes.get(node.id)!).filter((node) => node.y + node.height > grown.y + grown.height)
    const overlap = Math.max(0, ...below.map((node) => bottom + BAND_GAP - node.y))
    if (overlap > 0) shiftFrom(grown.y + grown.height + 0.5, overlap)
  }
  return { nodes, edges }
}

function boundsOf(boxes: readonly Box[], edges: readonly PlacedEdge[]): Box {
  const points = edges.flatMap((edge) => edge.points).map((point) => ({ x: point.x, y: point.y, width: 0, height: 0 }))
  return union([...boxes, ...points]) ?? { x: 0, y: 0, width: 0, height: 0 }
}
