import type { DiagramId, ProjectId, TaskId } from '@/model/branded'

/**
 * The Semantic Graph as the server stores it: what a system does, as typed nodes and typed
 * relations, each traceable to the code that implements it. The console only ever draws it;
 * nothing here is written by hand.
 */

/** The kinds the console draws a dedicated shape for. Any other snake_case kind is drawn generically. */
export const KNOWN_NODE_KINDS = [
  'concept',
  'action',
  'decision',
  'state',
  'event',
  'data',
  'external_system',
  'code'
] as const

export type KnownNodeKind = (typeof KNOWN_NODE_KINDS)[number]

/** A known kind, or a custom one a session introduced. */
export type NodeKind = KnownNodeKind | (string & { readonly __custom?: never })

export const KNOWN_RELATIONS = [
  'leads_to',
  'conditionally_leads_to',
  'transitions_to',
  'triggers',
  'calls',
  'reads',
  'writes',
  'produces',
  'consumes',
  'depends_on',
  'contains'
] as const

export type KnownRelation = (typeof KNOWN_RELATIONS)[number]

export type RelationKind = KnownRelation | (string & { readonly __custom?: never })

export type Provenance = 'observed' | 'inferred' | 'documented' | 'stated'

export type Metadata = Readonly<Record<string, unknown>>

export interface SourceLocation {
  readonly file: string
  readonly startLine: number | null
  readonly endLine: number | null
  readonly symbol: string | null
}

export interface GraphNode {
  readonly id: string
  readonly kind: NodeKind
  readonly title: string
  readonly description: string | null
  readonly sources: readonly SourceLocation[]
  readonly provenance: Provenance | null
  readonly confidence: number | null
  readonly metadata: Metadata
}

export interface GraphEdge {
  readonly id: string
  readonly from: string
  readonly to: string
  readonly relation: RelationKind
  readonly label: string | null
  readonly provenance: Provenance | null
  readonly confidence: number | null
  readonly metadata: Metadata
}

export interface SemanticGraph {
  readonly format: string
  readonly version: number
  readonly nodes: readonly GraphNode[]
  readonly edges: readonly GraphEdge[]
}

/** A diagram as the library lists it: everything but the graph. */
export interface DiagramSummary {
  readonly id: DiagramId
  readonly projectId: ProjectId
  readonly taskId: TaskId | null
  readonly title: string
  readonly question: string
  readonly nodeCount: number
  readonly edgeCount: number
  readonly createdAt: string
  readonly updatedAt: string
}

export interface Diagram extends DiagramSummary {
  readonly graph: SemanticGraph
}

export interface SourceLine {
  readonly number: number
  readonly text: string
}

/** A span of real code with a few lines around it; the highlight is the span itself. */
export interface SourceExcerpt {
  readonly file: string
  readonly language: string | null
  readonly highlightStart: number | null
  readonly highlightEnd: number | null
  readonly totalLines: number
  readonly lines: readonly SourceLine[]
  readonly truncated: boolean
}

export interface DiagramStreamEvent {
  readonly diagramId: DiagramId
  readonly diagram: DiagramSummary | null
  readonly deleted: boolean
}

/** What the console sends to store a graph it was handed (an import). */
export interface DiagramDraft {
  readonly projectId: ProjectId
  readonly taskId: TaskId | null
  readonly title: string
  readonly question: string
  readonly graph: unknown
}
