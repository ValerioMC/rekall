import { parseMermaid } from './mermaid-runtime'
import { readGraph } from './graph-reader'
import { readSequence } from './sequence-reader'
import type { GraphScene } from '../scene/graph-scene'
import type { SequenceScene } from '../scene/sequence-scene'
import type { DiagramSheet } from '@/model/diagram'

/**
 * A sheet read for drawing: the five types the console draws itself become scenes, every other
 * type is left to Mermaid's own renderer.
 */
export type SheetScene =
  | { readonly drawing: 'sequence'; readonly scene: SequenceScene }
  | { readonly drawing: 'graph'; readonly scene: GraphScene }
  | { readonly drawing: 'native' }

export async function readSheet(sheet: DiagramSheet): Promise<SheetScene> {
  switch (sheet.kind) {
    case 'sequence':
      return { drawing: 'sequence', scene: readSequence((await parseMermaid(sheet.mermaid)).db) }
    case 'flowchart':
    case 'state':
    case 'class':
    case 'er':
      return { drawing: 'graph', scene: readGraph(sheet.kind, (await parseMermaid(sheet.mermaid)).db) }
    default:
      return { drawing: 'native' }
  }
}
