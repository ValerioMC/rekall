import { callTreeOf, type CallNode } from './call-tree'
import { layoutGraph, type GraphLayout } from './layout/graph-layout'
import { layoutSequence, type SequenceLayout } from './layout/sequence-layout'
import { renderMermaid } from './mermaid/mermaid-runtime'
import { placeNativeSvg } from './mermaid/native-svg'
import { readSheet } from './mermaid/sheet-reader'
import type { GraphScene } from './scene/graph-scene'
import type { SequenceScene } from './scene/sequence-scene'
import type { Box } from './viewport'
import type { DiagramSheet } from '@/model/diagram'

/** A sheet ready to draw: its scene and geometry, or Mermaid's own SVG for the other types. */
export type SheetDrawing =
  | { readonly drawing: 'sequence'; readonly scene: SequenceScene; readonly layout: SequenceLayout; readonly calls: readonly CallNode[]; readonly bounds: Box }
  | { readonly drawing: 'graph'; readonly scene: GraphScene; readonly layout: GraphLayout; readonly bounds: Box }
  | { readonly drawing: 'native'; readonly svg: string; readonly bounds: Box }

export async function drawSheet(sheet: DiagramSheet): Promise<SheetDrawing> {
  const read = await readSheet(sheet)
  switch (read.drawing) {
    case 'sequence': {
      const layout = layoutSequence(read.scene)
      return { drawing: 'sequence', scene: read.scene, layout, calls: callTreeOf(read.scene), bounds: layout.bounds }
    }
    case 'graph': {
      const layout = layoutGraph(read.scene)
      return { drawing: 'graph', scene: read.scene, layout, bounds: layout.bounds }
    }
    case 'native': {
      const placed = placeNativeSvg(await renderMermaid(sheet.mermaid))
      return { drawing: 'native', svg: placed.markup, bounds: placed.bounds }
    }
  }
}
