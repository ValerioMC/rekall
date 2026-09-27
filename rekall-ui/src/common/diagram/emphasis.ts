/**
 * How strongly one element is drawn, given what the reader is looking at. `rest` when nothing
 * is being looked at; otherwise the element is the selection, lit around it, or dimmed.
 */
export type Emphasis = 'rest' | 'selected' | 'lit' | 'dimmed'

export interface Spotlight {
  readonly selected: string | null
  readonly nodes: ReadonlySet<string>
  readonly edges: ReadonlySet<string>
}

export function nodeEmphasis(spotlight: Spotlight | null, id: string): Emphasis {
  if (!spotlight) return 'rest'
  if (spotlight.selected === id) return 'selected'
  return spotlight.nodes.has(id) ? 'lit' : 'dimmed'
}

export function edgeEmphasis(spotlight: Spotlight | null, key: string): Emphasis {
  if (!spotlight) return 'rest'
  return spotlight.edges.has(key) ? 'lit' : 'dimmed'
}
