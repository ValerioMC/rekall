import type { DiagramKind } from '@/features/diagram/model/diagram'

/** The kinds the console draws with its own components. */
export type DrawnKind = Extract<DiagramKind, 'sequence' | 'flowchart' | 'state' | 'class' | 'er'>

const DECLARATIONS: ReadonlyMap<string, DrawnKind> = new Map([
  ['sequenceDiagram', 'sequence'],
  ['flowchart', 'flowchart'],
  ['graph', 'flowchart'],
  ['stateDiagram', 'state'],
  ['stateDiagram-v2', 'state'],
  ['classDiagram', 'class'],
  ['classDiagram-v2', 'class'],
  ['erDiagram', 'er']
])

const FRONT_MATTER = /^\s*---[\s\S]*?\n---[^\n]*(\n|$)/

/** The console-drawn kind a bare Mermaid source declares on its first line, or null when Mermaid draws it. */
export function drawnKindOf(source: string): DrawnKind | null {
  return DECLARATIONS.get(declarationOf(source)) ?? null
}

function declarationOf(source: string): string {
  const firstLine = source
    .replace(FRONT_MATTER, '')
    .split('\n')
    .map((line) => line.trim())
    .find((line) => line !== '' && !line.startsWith('%%'))
  return firstLine?.split(/\s+/)[0] ?? ''
}
