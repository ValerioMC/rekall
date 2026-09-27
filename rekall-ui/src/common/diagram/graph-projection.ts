import type { GraphEdge, GraphNode, RelationKind, SemanticGraph } from '@/model/diagram'

/**
 * Turns a graph into what is on screen, before any geometry: which nodes show, which are drawn
 * as clusters around the nodes they contain, which are folded, and which edges survive.
 *
 * Two lenses over the same graph: `concept` hides the code nodes, so a monolithic function's
 * conceptual pieces stand on their own; `code` keeps them and draws each one as a frame around
 * the pieces it implements. A collapsed node absorbs everything it contains, and edges from
 * inside it are re-attached to it.
 */

export type Lens = 'concept' | 'code'

export interface ProjectionOptions {
  readonly lens: Lens
  readonly collapsed: ReadonlySet<string>
}

export interface ProjectedNode {
  readonly id: string
  readonly node: GraphNode
  readonly parentId: string | null
  /** Drawn as a frame around its visible children. */
  readonly isCluster: boolean
  /** Folded, standing in for `hiddenCount` descendants. */
  readonly collapsed: boolean
  readonly hiddenCount: number
  /** Can be folded or unfolded: it contains something visible under this lens. */
  readonly foldable: boolean
}

export interface ProjectedEdge {
  readonly key: string
  readonly from: string
  readonly to: string
  readonly relation: RelationKind
  readonly label: string | null
  /** The graph edges this one stands for; several once folding merges them. */
  readonly edgeIds: readonly string[]
}

export interface Projection {
  readonly nodes: readonly ProjectedNode[]
  readonly edges: readonly ProjectedEdge[]
}

export function containerMap(graph: SemanticGraph): ReadonlyMap<string, string> {
  const parents = new Map<string, string>()
  for (const edge of graph.edges) {
    if (edge.relation === 'contains') parents.set(edge.to, edge.from)
  }
  return parents
}

export function project(graph: SemanticGraph, options: ProjectionOptions): Projection {
  const byId = new Map(graph.nodes.map((node) => [node.id, node]))
  const parents = containerMap(graph)
  const hiddenByLens = (id: string): boolean => options.lens === 'concept' && byId.get(id)?.kind === 'code'

  /** The nearest container still visible under the lens. */
  const visibleParent = (id: string): string | null => {
    let parent = parents.get(id) ?? null
    while (parent !== null && hiddenByLens(parent)) parent = parents.get(parent) ?? null
    return parent
  }

  /** What stands for `id` on screen: itself, its outermost collapsed ancestor, or nothing. */
  const representative = (id: string): string | null => {
    if (hiddenByLens(id) || !byId.has(id)) return null
    let stand = id
    let ancestor = visibleParent(id)
    while (ancestor !== null) {
      if (options.collapsed.has(ancestor)) stand = ancestor
      ancestor = visibleParent(ancestor)
    }
    return stand
  }

  const shown = graph.nodes.filter((node) => representative(node.id) === node.id)
  const visibleChildren = new Map<string, number>()
  const folded = new Map<string, number>()
  for (const node of graph.nodes) {
    const stand = representative(node.id)
    if (stand === null) continue
    const parent = visibleParent(node.id)
    if (parent !== null) visibleChildren.set(parent, (visibleChildren.get(parent) ?? 0) + 1)
    if (stand !== node.id) folded.set(stand, (folded.get(stand) ?? 0) + 1)
  }

  const nodes: ProjectedNode[] = shown.map((node) => {
    const foldable = (visibleChildren.get(node.id) ?? 0) > 0
    const collapsed = foldable && options.collapsed.has(node.id)
    return {
      id: node.id,
      node,
      parentId: visibleParent(node.id),
      isCluster: foldable && !collapsed,
      collapsed,
      hiddenCount: folded.get(node.id) ?? 0,
      foldable
    }
  })

  return { nodes, edges: projectEdges(graph.edges, representative) }
}

function projectEdges(
  edges: readonly GraphEdge[],
  representative: (id: string) => string | null
): ProjectedEdge[] {
  const merged = new Map<string, { from: string; to: string; relation: RelationKind; label: string | null; edgeIds: string[] }>()
  for (const edge of edges) {
    if (edge.relation === 'contains') continue
    const from = representative(edge.from)
    const to = representative(edge.to)
    if (from === null || to === null || from === to) continue
    const key = `${from}\u0000${to}\u0000${edge.relation}\u0000${edge.label ?? ''}`
    const existing = merged.get(key)
    if (existing) existing.edgeIds.push(edge.id)
    else merged.set(key, { from, to, relation: edge.relation, label: edge.label, edgeIds: [edge.id] })
  }
  return [...merged.entries()].map(([key, edge]) => ({ key, ...edge }))
}
