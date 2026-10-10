import type { DiagramKind } from '@/model/diagram'

/** What each Mermaid type is called, what it is for, and whether the console draws it itself. */
export interface KindInfo {
  readonly label: string
  readonly purpose: string
  /** Drawn with the console's own components, so its elements can open their code. */
  readonly drawnHere: boolean
}

export const KIND_INFO: Readonly<Record<DiagramKind, KindInfo>> = {
  sequence: { label: 'Sequence', purpose: 'Who calls whom, in order', drawnHere: true },
  flowchart: { label: 'Flowchart', purpose: 'Steps and decisions in one piece of code', drawnHere: true },
  state: { label: 'State', purpose: 'A lifecycle and what moves it', drawnHere: true },
  class: { label: 'Class', purpose: 'Types, members and how they relate', drawnHere: true },
  er: { label: 'Entity–relationship', purpose: 'Tables and their keys', drawnHere: true },
  gantt: { label: 'Gantt', purpose: 'Work over time', drawnHere: false },
  pie: { label: 'Pie', purpose: 'Shares of a whole', drawnHere: false },
  mindmap: { label: 'Mindmap', purpose: 'A topic broken into parts', drawnHere: false },
  timeline: { label: 'Timeline', purpose: 'Events in order', drawnHere: false },
  journey: { label: 'User journey', purpose: 'What a user goes through', drawnHere: false },
  quadrant: { label: 'Quadrant', purpose: 'Items on two axes', drawnHere: false },
  gitgraph: { label: 'Git graph', purpose: 'Branches and merges', drawnHere: false },
  requirement: { label: 'Requirement', purpose: 'Requirements and what satisfies them', drawnHere: false },
  c4: { label: 'C4', purpose: 'Systems, containers and components', drawnHere: false },
  sankey: { label: 'Sankey', purpose: 'Flows between quantities', drawnHere: false },
  xychart: { label: 'XY chart', purpose: 'Values on two axes', drawnHere: false },
  block: { label: 'Block', purpose: 'Blocks laid out on a grid', drawnHere: false },
  packet: { label: 'Packet', purpose: 'A binary layout', drawnHere: false },
  architecture: { label: 'Architecture', purpose: 'Services and their connections', drawnHere: false },
  kanban: { label: 'Kanban', purpose: 'Work by column', drawnHere: false },
  radar: { label: 'Radar', purpose: 'Scores on several axes', drawnHere: false }
}

/** What the generate dialog offers: the types that read code well, the drawn-here ones first. */
export const OFFERED_KINDS: readonly DiagramKind[] = [
  'sequence',
  'flowchart',
  'state',
  'class',
  'er',
  'c4',
  'architecture',
  'mindmap',
  'requirement',
  'timeline'
]
