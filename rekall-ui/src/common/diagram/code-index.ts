import type { GraphNode, SemanticGraph, SourceLocation } from '@/model/diagram'

/**
 * CODE → CONCEPT inside one diagram: the files its elements point at, and for each file which
 * elements live there. The server answers the same question across every diagram of a project.
 */

export interface FileTrace {
  readonly file: string
  readonly nodeIds: readonly string[]
}

export function normalisePath(path: string): string {
  return path.trim().replace(/\\/g, '/').replace(/^(\.\/)+/, '')
}

export function filesOf(graph: SemanticGraph): FileTrace[] {
  const byFile = new Map<string, Set<string>>()
  for (const node of graph.nodes) {
    for (const source of node.sources) {
      const file = normalisePath(source.file)
      const ids = byFile.get(file) ?? new Set<string>()
      ids.add(node.id)
      byFile.set(file, ids)
    }
  }
  return [...byFile.entries()]
    .map(([file, ids]) => ({ file, nodeIds: [...ids] }))
    .sort((a, b) => b.nodeIds.length - a.nodeIds.length || a.file.localeCompare(b.file))
}

export interface Coverage {
  readonly traced: number
  readonly total: number
}

/** How many elements can be followed to code at all. */
export function coverageOf(graph: SemanticGraph): Coverage {
  return { traced: graph.nodes.filter((node) => node.sources.length > 0).length, total: graph.nodes.length }
}

/** `src/order.rs` · `L12–30` · the short form a source is listed under. */
export function spanLabel(source: SourceLocation): string {
  if (source.startLine === null) return 'whole file'
  if (source.endLine === null || source.endLine === source.startLine) return `L${source.startLine}`
  return `L${source.startLine}–${source.endLine}`
}

export function isTraced(node: GraphNode): boolean {
  return node.sources.length > 0
}

/** `src/pty/manager.rs` → `src/pty/`: shown dimmed and cut first, so the file name survives. */
export function folderOf(path: string): string {
  const at = path.lastIndexOf('/')
  return at < 0 ? '' : path.slice(0, at + 1)
}

export function baseOf(path: string): string {
  return path.slice(path.lastIndexOf('/') + 1)
}
