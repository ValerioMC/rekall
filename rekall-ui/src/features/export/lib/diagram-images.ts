import { mermaidFenceSources } from '@/common/markdown/mermaid-fence'
import type { DiagramFailure, DiagramImage, DiagramProgress } from '@/features/export/model/diagram-image'

export type DiagramRenderer = (source: string, scale: number) => Promise<Uint8Array>

const SHARP_SCALE = 2
const PLAIN_SCALE = 1
const MAX_IMAGE_BYTES = 2 * 1024 * 1024
const MAX_TOTAL_BYTES = 48 * 1024 * 1024

export function mermaidSourcesOf(markdown: string): string[] {
  const sources = mermaidFenceSources(markdown).map((source) => source.trim())
  return [...new Set(sources.filter((source) => source !== ''))]
}

/**
 * Draws each source on the paper palette and returns its PNG. A source that cannot be drawn is
 * left out and reported through `onProgress`: the server then writes the error paragraph in its place.
 * Aborting rejects with the signal's reason.
 */
export async function drawDiagramImages(
  sources: readonly string[],
  signal: AbortSignal,
  onProgress: (progress: DiagramProgress) => void,
  render: DiagramRenderer = renderInBrowser
): Promise<DiagramImage[]> {
  const images: DiagramImage[] = []
  const failures: DiagramFailure[] = []
  let sentBytes = 0
  for (const source of sources) {
    signal.throwIfAborted()
    try {
      const png = await smallEnough(source, render)
      if (sentBytes + png.length > MAX_TOTAL_BYTES) throw new Error('The diagrams are too large to send together.')
      sentBytes += png.length
      images.push({ source, png })
    } catch (failure: unknown) {
      signal.throwIfAborted()
      failures.push({ source, reason: failure instanceof Error ? failure.message : String(failure) })
    }
    onProgress({ finished: images.length + failures.length, total: sources.length, failures: [...failures] })
  }
  return images
}

async function smallEnough(source: string, render: DiagramRenderer): Promise<Uint8Array> {
  const sharp = await render(source, SHARP_SCALE)
  return sharp.length > MAX_IMAGE_BYTES ? render(source, PLAIN_SCALE) : sharp
}

async function renderInBrowser(source: string, scale: number): Promise<Uint8Array> {
  const { renderDiagramPng } = await import('./diagram-png')
  return renderDiagramPng(source, scale)
}
