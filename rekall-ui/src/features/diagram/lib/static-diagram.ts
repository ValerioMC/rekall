import { createApp } from 'vue'
import StaticDiagram from '@/features/diagram/components/static/StaticDiagram.vue'
import { drawSource, type SheetDrawing } from './sheet-drawing'

/** Drawing is slow and a page that redraws itself asks again for the same source, so each source is drawn once. */
const drawn = new Map<string, Promise<SheetDrawing>>()

/** Draws a Mermaid source into `host` as the console draws a sheet, and takes it down when `signal` aborts. */
export async function showStaticDiagram(host: HTMLElement, source: string, signal: AbortSignal): Promise<void> {
  const drawing = await drawingOf(source)
  if (signal.aborted) return
  const app = createApp(StaticDiagram, { drawing })
  app.mount(host)
  signal.addEventListener('abort', () => app.unmount(), { once: true })
}

function drawingOf(source: string): Promise<SheetDrawing> {
  const cached = drawn.get(source)
  if (cached) return cached
  const drawing = drawSource(source)
  drawing.catch(() => drawn.delete(source))
  drawn.set(source, drawing)
  return drawing
}
