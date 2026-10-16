import type { NodeOutline } from './scene/graph-scene'

/**
 * The outline of each card in a `width × height` box at the origin, and the one inner detail
 * some outlines carry (a subroutine's bars, a data store's rim, a note's fold). Every outline
 * keeps the card's footprint, so a decision still reads as a card with pointed ends.
 */

const POINT = 16
const CYLINDER_RIM = 7
const SUBROUTINE_BAR = 9
const FOLD = 12

export function shapePath(outline: NodeOutline, width: number, height: number): string {
  switch (outline) {
    case 'rounded':
      return roundedRect(0, 0, width, height, 12)
    case 'stadium':
    case 'circle':
      return roundedRect(0, 0, width, height, height / 2)
    case 'diamond':
      return `M${POINT},0H${width - POINT}L${width},${height / 2}L${width - POINT},${height}H${POINT}L0,${height / 2}Z`
    case 'hexagon':
      return `M${POINT / 2},0H${width - POINT / 2}L${width},${height / 2}L${width - POINT / 2},${height}H${POINT / 2}L0,${height / 2}Z`
    case 'slant':
      return `M${POINT},0H${width}L${width - POINT},${height}H0Z`
    case 'flag':
      return `M0,0H${width - 8}Q${width},0 ${width},8V${height - 8}Q${width},${height} ${width - 8},${height}H0L${POINT},${height / 2}Z`
    case 'cylinder': {
      const rx = width / 2
      return `M0,${CYLINDER_RIM}A${rx},${CYLINDER_RIM} 0 0 1 ${width},${CYLINDER_RIM}V${height - CYLINDER_RIM}A${rx},${CYLINDER_RIM} 0 0 1 0,${height - CYLINDER_RIM}Z`
    }
    case 'note':
      return `M0,0H${width - FOLD}L${width},${FOLD}V${height}H0Z`
    default:
      return roundedRect(0, 0, width, height, 6)
  }
}

export function detailPath(outline: NodeOutline, width: number, height: number): string | null {
  switch (outline) {
    case 'circle':
      return roundedRect(3.5, 3.5, width - 7, height - 7, (height - 7) / 2)
    case 'cylinder':
      return `M0,${CYLINDER_RIM}A${width / 2},${CYLINDER_RIM} 0 0 0 ${width},${CYLINDER_RIM}`
    case 'subroutine':
      return `M${SUBROUTINE_BAR},0V${height}M${width - SUBROUTINE_BAR},0V${height}`
    case 'note':
      return `M${width - FOLD},0V${FOLD}H${width}`
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
