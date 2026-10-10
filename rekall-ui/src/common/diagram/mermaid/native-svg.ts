import type { Box } from '../viewport'

/**
 * Mermaid's SVG made to sit in the canvas world: its responsive width and max-width style give
 * way to the size its view box declares, which is also the box the camera fits.
 */
export interface NativeSvg {
  readonly markup: string
  readonly bounds: Box
}

export function placeNativeSvg(svg: string): NativeSvg {
  const document = new DOMParser().parseFromString(svg, 'image/svg+xml')
  const root = document.documentElement
  const [x, y, width, height] = (root.getAttribute('viewBox') ?? '0 0 0 0').split(/[\s,]+/).map(Number)
  const bounds = { x: 0, y: 0, width: width || 0, height: height || 0 }
  root.setAttribute('width', String(bounds.width))
  root.setAttribute('height', String(bounds.height))
  root.setAttribute('viewBox', `${x || 0} ${y || 0} ${bounds.width} ${bounds.height}`)
  root.removeAttribute('style')
  return { markup: new XMLSerializer().serializeToString(root), bounds }
}
