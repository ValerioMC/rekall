import type { Projection } from './graph-projection'

/**
 * What stays lit around a selection: the node, everything one relation away, and the frames
 * it sits in or holds. Everything else dims, so a dense diagram reads as one question at a time.
 */
export interface Neighbourhood {
  readonly nodes: ReadonlySet<string>
  readonly edges: ReadonlySet<string>
}

export function neighbourhoodOf(projection: Projection, id: string): Neighbourhood {
  const nodes = new Set<string>([id])
  const edges = new Set<string>()
  for (const edge of projection.edges) {
    if (edge.from === id || edge.to === id) {
      edges.add(edge.key)
      nodes.add(edge.from)
      nodes.add(edge.to)
    }
  }
  for (const node of projection.nodes) {
    if (node.parentId === id) nodes.add(node.id)
  }
  let parent = projection.nodes.find((node) => node.id === id)?.parentId ?? null
  while (parent !== null) {
    nodes.add(parent)
    const current: string = parent
    parent = projection.nodes.find((node) => node.id === current)?.parentId ?? null
  }
  return { nodes, edges }
}

/** A set of nodes lit as a group (a file's elements, a kind), with the edges between them. */
export function groupOf(projection: Projection, ids: ReadonlySet<string>): Neighbourhood {
  const edges = new Set(
    projection.edges.filter((edge) => ids.has(edge.from) && ids.has(edge.to)).map((edge) => edge.key)
  )
  return { nodes: ids, edges }
}
