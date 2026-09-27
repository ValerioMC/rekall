import type { NodeShape } from './kind-style'

/**
 * The outline of each node shape in a `width × height` box at the origin, and the one inner
 * detail some shapes carry (a state's second ring, a data store's rim, a subsystem's bars).
 * Pure strings, so the canvas and the legend draw the same outline.
 */

const LOZENGE_POINT = 16
const SIGNAL_POINT = 16
const CYLINDER_RIM = 7
const SUBSYSTEM_BAR = 10

export function shapePath(shape: NodeShape, width: number, height: number): string {
  switch (shape) {
    case 'soft':
      return roundedRect(0, 0, width, height, 18)
    case 'rounded':
      return roundedRect(0, 0, width, height, 10)
    case 'block':
      return roundedRect(0, 0, width, height, 6)
    case 'stadium':
      return roundedRect(0, 0, width, height, height / 2)
    case 'subsystem':
      return roundedRect(0, 0, width, height, 6)
    case 'lozenge': {
      const p = LOZENGE_POINT
      return `M${p},0H${width - p}L${width},${height / 2}L${width - p},${height}H${p}L0,${height / 2}Z`
    }
    case 'signal': {
      const r = 8
      const p = SIGNAL_POINT
      return `M${r},0H${width - p}L${width},${height / 2}L${width - p},${height}H${r}Q0,${height} 0,${height - r}V${r}Q0,0 ${r},0Z`
    }
    case 'cylinder': {
      const e = CYLINDER_RIM
      const rx = width / 2
      return `M0,${e}A${rx},${e} 0 0 1 ${width},${e}V${height - e}A${rx},${e} 0 0 1 0,${height - e}Z`
    }
  }
}

export function detailPath(shape: NodeShape, width: number, height: number): string | null {
  switch (shape) {
    case 'stadium':
      return roundedRect(3.5, 3.5, width - 7, height - 7, (height - 7) / 2)
    case 'cylinder':
      return `M0,${CYLINDER_RIM}A${width / 2},${CYLINDER_RIM} 0 0 0 ${width},${CYLINDER_RIM}`
    case 'subsystem':
      return `M${SUBSYSTEM_BAR},0V${height}M${width - SUBSYSTEM_BAR},0V${height}`
    default:
      return null
  }
}

export function roundedRect(x: number, y: number, width: number, height: number, radius: number): string {
  const r = Math.max(0, Math.min(radius, width / 2, height / 2))
  return (
    `M${x + r},${y}H${x + width - r}A${r},${r} 0 0 1 ${x + width},${y + r}` +
    `V${y + height - r}A${r},${r} 0 0 1 ${x + width - r},${y + height}` +
    `H${x + r}A${r},${r} 0 0 1 ${x},${y + height - r}V${y + r}A${r},${r} 0 0 1 ${x + r},${y}Z`
  )
}
