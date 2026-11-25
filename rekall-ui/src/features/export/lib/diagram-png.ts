import { createApp, nextTick } from 'vue'
import StaticDiagram from '@/features/diagram/components/static/StaticDiagram.vue'
import { PAPER_TOKENS } from '@/features/diagram/lib/diagram-palette'
import { drawingOf } from '@/features/diagram/lib/static-diagram'

const PADDING_PX = 16
const MAX_SIDE_PX = 8192
const WHITE = '#ffffff'
const PAPER_FRAME_CLASS = 'rk-paper-frame'

const PAINTED_PROPERTIES = [
  'fill', 'fill-opacity', 'stroke', 'stroke-width', 'stroke-opacity', 'stroke-dasharray', 'stroke-linecap',
  'stroke-linejoin', 'opacity', 'color', 'font-family', 'font-size', 'font-weight', 'font-style', 'letter-spacing',
  'text-anchor', 'text-decoration', 'dominant-baseline', 'paint-order', 'visibility', 'display'
] as const

/**
 * One source drawn on the paper palette in an off-screen frame that carries the paper tokens, so the
 * console's own drawings resolve light, then turned into a PNG with a white ground. A canvas loses
 * stylesheet rules, so what the drawing computed is written onto each element first.
 */
export async function renderDiagramPng(source: string, scale: number): Promise<Uint8Array> {
  const drawing = await drawingOf(source, 'paper')
  const frame = createPaperFrame()
  const host = document.createElement('div')
  frame.append(host)
  const app = createApp(StaticDiagram, { drawing })
  try {
    app.mount(host)
    await nextTick()
    const svg = host.querySelector('svg')
    if (!svg) throw new Error('The drawing produced no image.')
    const { width, height } = drawing.bounds
    return await pngOf(paintedCopy(svg), width, height, scale)
  } finally {
    app.unmount()
    frame.remove()
  }
}

function createPaperFrame(): HTMLElement {
  const frame = document.createElement('div')
  frame.setAttribute('aria-hidden', 'true')
  frame.className = PAPER_FRAME_CLASS
  Object.assign(frame.style, { position: 'fixed', left: '-100000px', top: '0', pointerEvents: 'none' })
  for (const [token, value] of Object.entries(PAPER_TOKENS)) frame.style.setProperty(token, value)
  const still = document.createElement('style')
  still.textContent = `.${PAPER_FRAME_CLASS} * { animation: none !important; transition: none !important; }`
  frame.append(still)
  document.body.append(frame)
  return frame
}

function paintedCopy(live: SVGSVGElement): SVGSVGElement {
  const copy = live.cloneNode(true) as SVGSVGElement
  writeComputedStyles(live, copy)
  removeComments(copy)
  return copy
}

function removeComments(root: Node): void {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_COMMENT)
  const comments: Node[] = []
  while (walker.nextNode()) comments.push(walker.currentNode)
  comments.forEach((comment) => comment.parentNode?.removeChild(comment))
}

function writeComputedStyles(live: Element, copy: Element): void {
  const computed = getComputedStyle(live)
  const painted = PAINTED_PROPERTIES.map((property) => `${property}:${computed.getPropertyValue(property)}`).join(';')
  copy.setAttribute('style', `${copy.getAttribute('style') ?? ''};${painted}`)
  Array.from(live.children).forEach((child, index) => {
    const twin = copy.children[index]
    if (twin) writeComputedStyles(child, twin)
  })
}

async function pngOf(svg: SVGSVGElement, width: number, height: number, scale: number): Promise<Uint8Array> {
  const fitted = Math.min(scale, MAX_SIDE_PX / (Math.max(width, height) + 2 * PADDING_PX))
  const canvas = document.createElement('canvas')
  canvas.width = Math.ceil((width + 2 * PADDING_PX) * fitted)
  canvas.height = Math.ceil((height + 2 * PADDING_PX) * fitted)
  const context = canvas.getContext('2d')
  if (!context) throw new Error('The browser gave no canvas to draw on.')
  context.fillStyle = WHITE
  context.fillRect(0, 0, canvas.width, canvas.height)
  const image = await loadSvg(svg)
  context.drawImage(image, PADDING_PX * fitted, PADDING_PX * fitted, width * fitted, height * fitted)
  const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, 'image/png'))
  if (!blob) throw new Error('The canvas gave no image.')
  return new Uint8Array(await blob.arrayBuffer())
}

async function loadSvg(svg: SVGSVGElement): Promise<HTMLImageElement> {
  const markup = new XMLSerializer().serializeToString(svg)
  const image = new Image()
  image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(markup)}`
  await image.decode()
  return image
}
