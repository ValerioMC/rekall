import type { DiagramKind } from '@/model/diagram'

/**
 * A flowchart, state, class or ER sheet as the console draws it: boxes, groups and the lines
 * between them, read out of Mermaid and free of its rendering. Layout and drawing work on this.
 */

export type GraphDirection = 'TB' | 'BT' | 'LR' | 'RL'

/** What an element is to a reader, which decides its tint and glyph. */
export type ElementRole =
  | 'step'
  | 'decision'
  | 'terminal'
  | 'data'
  | 'state'
  | 'pseudo'
  | 'class'
  | 'entity'
  | 'note'
  | 'participant'
  | 'actor'

export type NodeOutline =
  | 'rect'
  | 'rounded'
  | 'stadium'
  | 'subroutine'
  | 'diamond'
  | 'hexagon'
  | 'circle'
  | 'cylinder'
  | 'slant'
  | 'flag'
  | 'start'
  | 'end'
  | 'bar'
  | 'note'

/** One line inside a class or entity box: `+place(order) Result`, `string id PK`. */
export interface CompartmentRow {
  readonly text: string
  readonly detail: string | null
}

export interface GraphNode {
  readonly id: string
  /** The key a code link addresses it by: its Mermaid id, or an entity's name. */
  readonly linkKey: string
  readonly label: string
  readonly role: ElementRole
  readonly outline: NodeOutline
  readonly parentId: string | null
  /** «interface», «enum»: shown above the name. */
  readonly annotations: readonly string[]
  /** Attributes then methods for a class, attributes for an entity; empty for the rest. */
  readonly compartments: readonly (readonly CompartmentRow[])[]
}

export interface GraphGroup {
  readonly id: string
  readonly label: string
  readonly parentId: string | null
}

export type EdgeLine = 'solid' | 'dashed' | 'dotted' | 'thick'

/** What sits on an end of a line: arrows, UML markers and ER cardinalities. */
export type EdgeEnd =
  | 'none'
  | 'arrow'
  | 'cross'
  | 'circle'
  | 'triangle'
  | 'diamond'
  | 'diamond-filled'
  | 'open-arrow'
  | 'one'
  | 'zero-or-one'
  | 'one-or-more'
  | 'zero-or-more'

export interface GraphEdge {
  readonly key: string
  readonly from: string
  readonly to: string
  readonly label: string | null
  readonly line: EdgeLine
  readonly startEnd: EdgeEnd
  readonly endEnd: EdgeEnd
  /** A class relation's cardinalities, written beside each end. */
  readonly startLabel: string | null
  readonly endLabel: string | null
}

export interface GraphScene {
  readonly kind: Extract<DiagramKind, 'flowchart' | 'state' | 'class' | 'er'>
  readonly direction: GraphDirection
  readonly nodes: readonly GraphNode[]
  readonly groups: readonly GraphGroup[]
  readonly edges: readonly GraphEdge[]
}
