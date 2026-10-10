import type { EdgeEnd } from '@/common/diagram/scene/graph-scene'

/**
 * Each end marker drawn pointing along +x with its tip at the origin, as the line arrives from
 * the left; `inset` is how far the line stops short so the marker sits on the border.
 */
export interface EdgeMarker {
  readonly d: string
  readonly fill: 'line' | 'canvas' | 'none'
  readonly inset: number
}

const CROW_FOOT = 'M0,-6L-9,0L0,6M-9,0H0'

const MARKERS: Readonly<Record<Exclude<EdgeEnd, 'none'>, EdgeMarker>> = {
  arrow: { d: 'M0,0L-8.5,-4.4L-6.4,0L-8.5,4.4Z', fill: 'line', inset: 7 },
  'open-arrow': { d: 'M-8,-4.5L0,0L-8,4.5', fill: 'none', inset: 0 },
  triangle: { d: 'M0,0L-11,-6.5L-11,6.5Z', fill: 'canvas', inset: 11 },
  diamond: { d: 'M0,0L-7,-5L-14,0L-7,5Z', fill: 'canvas', inset: 14 },
  'diamond-filled': { d: 'M0,0L-7,-5L-14,0L-7,5Z', fill: 'line', inset: 14 },
  cross: { d: 'M-8,-4L-2,4M-8,4L-2,-4', fill: 'none', inset: 0 },
  circle: { d: 'M0,0A4.5,4.5 0 1 1 -9,0A4.5,4.5 0 1 1 0,0Z', fill: 'canvas', inset: 9 },
  one: { d: 'M-6,-6V6M-10,-6V6', fill: 'none', inset: 0 },
  'zero-or-one': { d: 'M-6,-6V6M-9,0A3.5,3.5 0 1 1 -16,0A3.5,3.5 0 1 1 -9,0Z', fill: 'canvas', inset: 0 },
  'one-or-more': { d: `${CROW_FOOT}M-12,-6V6`, fill: 'none', inset: 0 },
  'zero-or-more': { d: `${CROW_FOOT}M-11,0A3.5,3.5 0 1 1 -18,0A3.5,3.5 0 1 1 -11,0Z`, fill: 'canvas', inset: 0 }
}

export function markerOf(end: EdgeEnd): EdgeMarker | null {
  return end === 'none' ? null : MARKERS[end]
}
