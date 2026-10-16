import type { DiagramId, ProjectId, TaskId } from '@/model/branded'

/**
 * A diagram as the server stores it: Mermaid sheets a session wrote, each tying the elements it
 * draws to the code that implements them. The console only ever draws it.
 */

export const DIAGRAM_KINDS = [
  'sequence',
  'flowchart',
  'state',
  'class',
  'er',
  'gantt',
  'pie',
  'mindmap',
  'timeline',
  'journey',
  'quadrant',
  'gitgraph',
  'requirement',
  'c4',
  'sankey',
  'xychart',
  'block',
  'packet',
  'architecture',
  'kanban',
  'radar'
] as const

export type DiagramKind = (typeof DIAGRAM_KINDS)[number]

export interface SourceLocation {
  readonly file: string
  readonly startLine: number | null
  readonly endLine: number | null
  readonly symbol: string | null
}

/** Element key (a Mermaid id, or `#n` for a sequence's n-th message) to where it lives. */
export type CodeLinks = Readonly<Record<string, readonly SourceLocation[]>>

export interface DiagramSheet {
  readonly kind: DiagramKind
  readonly title: string | null
  readonly mermaid: string
  readonly links: CodeLinks
}

/** A diagram as the gallery lists it: everything but its sheets. */
export interface DiagramSummary {
  readonly id: DiagramId
  readonly projectId: ProjectId
  readonly taskId: TaskId | null
  readonly title: string
  readonly question: string
  readonly kinds: readonly DiagramKind[]
  readonly createdAt: string
  readonly updatedAt: string
}

export interface Diagram extends DiagramSummary {
  readonly sheets: readonly DiagramSheet[]
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

/** What the console sends to store Mermaid it was handed (an import). */
export interface DiagramDraft {
  readonly projectId: ProjectId
  readonly taskId: TaskId | null
  readonly title: string
  readonly question: string
  readonly sheets: readonly { readonly title: string | null; readonly mermaid: string }[]
}
