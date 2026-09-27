import type { Point } from './viewport'

/**
 * An SVG path through an edge's route points, smoothed as a Catmull-Rom spline so a layered
 * route reads as one gesture rather than a polyline with kinks. It stops short of the last
 * point by `inset` so the arrowhead, drawn separately, sits on the target's border.
 */
export function smoothPath(points: readonly Point[], inset = 0): string {
  if (points.length < 2) return ''
  const route = inset > 0 ? shortenEnd(points, inset) : [...points]
  const [first] = route
  if (!first) return ''
  if (route.length === 2) return `M${fmt(first)}L${fmt(route[1]!)}`
  let path = `M${fmt(first)}`
  for (let index = 0; index < route.length - 1; index += 1) {
    const p0 = route[index - 1] ?? route[index]!
    const p1 = route[index]!
    const p2 = route[index + 1]!
    const p3 = route[index + 2] ?? p2
    const c1 = { x: p1.x + (p2.x - p0.x) / 6, y: p1.y + (p2.y - p0.y) / 6 }
    const c2 = { x: p2.x - (p3.x - p1.x) / 6, y: p2.y - (p3.y - p1.y) / 6 }
    path += `C${fmt(c1)} ${fmt(c2)} ${fmt(p2)}`
  }
  return path
}

/** The angle, in degrees, the route arrives at its last point with. */
export function arrivalAngle(points: readonly Point[]): number {
  const last = points[points.length - 1]
  const before = points[points.length - 2]
  if (!last || !before) return 0
  return (Math.atan2(last.y - before.y, last.x - before.x) * 180) / Math.PI
}

/** The point halfway along the route by length: where an unlabelled edge's hover label goes. */
export function midpointAlong(points: readonly Point[]): Point | null {
  if (points.length === 0) return null
  const lengths = points.slice(1).map((point, index) => distance(points[index]!, point))
  let remaining = lengths.reduce((sum, length) => sum + length, 0) / 2
  for (let index = 0; index < lengths.length; index += 1) {
    const length = lengths[index]!
    if (remaining <= length && length > 0) {
      const from = points[index]!
      const to = points[index + 1]!
      const t = remaining / length
      return { x: from.x + (to.x - from.x) * t, y: from.y + (to.y - from.y) * t }
    }
    remaining -= length
  }
  return points[points.length - 1] ?? null
}

function shortenEnd(points: readonly Point[], inset: number): Point[] {
  const route = [...points]
  const last = route[route.length - 1]!
  const before = route[route.length - 2]!
  const length = distance(before, last)
  if (length <= inset) return route
  const t = (length - inset) / length
  route[route.length - 1] = { x: before.x + (last.x - before.x) * t, y: before.y + (last.y - before.y) * t }
  return route
}

function distance(a: Point, b: Point): number {
  return Math.hypot(b.x - a.x, b.y - a.y)
}

function fmt(point: Point): string {
  return `${point.x.toFixed(1)},${point.y.toFixed(1)}`
}
