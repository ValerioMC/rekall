import { callTreeOf, type CallNode } from './call-tree'
import { layoutGraph, type GraphLayout } from './layout/graph-layout'
import { layoutSequence, type SequenceLayout } from './layout/sequence-layout'
import { renderMermaid } from './mermaid/mermaid-runtime'
import { placeNativeSvg } from './mermaid/native-svg'
import { readSheet } from './mermaid/sheet-reader'
import { drawnKindOf } from './mermaid/source-kind'
import type { GraphScene } from './scene/graph-scene'
import type { SequenceScene } from './scene/sequence-scene'
import type { Box } from './viewport'
import type { DiagramSheet } from '@/features/diagram/model/diagram'

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
    case 'native':
      return drawNative(sheet.mermaid)
  }
}

/** A bare Mermaid source drawn the way a sheet of its kind is. */
export async function drawSource(source: string): Promise<SheetDrawing> {
  const kind = drawnKindOf(source)
  if (!kind) return drawNative(source)
  return drawSheet({ kind, title: null, mermaid: source, links: {}, walkthrough: null })
}

async function drawNative(source: string): Promise<SheetDrawing> {
  const placed = placeNativeSvg(await renderMermaid(source))
  return { drawing: 'native', svg: placed.markup, bounds: placed.bounds }
}
