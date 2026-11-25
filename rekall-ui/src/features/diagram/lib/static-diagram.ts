import { createApp } from 'vue'
import DiagramFrame from '@/features/diagram/components/static/DiagramFrame.vue'
import StaticDiagram from '@/features/diagram/components/static/StaticDiagram.vue'
import type { DiagramPalette } from './diagram-palette'
import { drawSource, type SheetDrawing } from './sheet-drawing'

/** Drawing is slow and a page that redraws itself asks again for the same source, so each source is drawn once per palette. */
const drawn = new Map<string, Promise<SheetDrawing>>()

/** Draws a Mermaid source into `host` as the console draws a sheet, and takes it down when `signal` aborts. `enlargeable` adds the way to open it over the window. */
export async function showStaticDiagram(host: HTMLElement, source: string, signal: AbortSignal, enlargeable = false): Promise<void> {
  const drawing = await drawingOf(source)
  if (signal.aborted) return
  const app = createApp(enlargeable ? DiagramFrame : StaticDiagram, { drawing })
  app.mount(host)
  signal.addEventListener('abort', () => app.unmount(), { once: true })
}

export function drawingOf(source: string, palette: DiagramPalette = 'console'): Promise<SheetDrawing> {
  const key = `${palette}\n${source}`
  const cached = drawn.get(key)
  if (cached) return cached
  const drawing = drawSource(source, palette)
  drawing.catch(() => drawn.delete(key))
  drawn.set(key, drawing)
  return drawing
}
