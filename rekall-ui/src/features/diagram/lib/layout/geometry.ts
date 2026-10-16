import type { Box, Point } from '../viewport'

/** Box arithmetic the layouts share. */

export function union(boxes: readonly Box[]): Box | null {
  if (boxes.length === 0) return null
  const left = Math.min(...boxes.map((box) => box.x))
  const top = Math.min(...boxes.map((box) => box.y))
  const right = Math.max(...boxes.map((box) => box.x + box.width))
  const bottom = Math.max(...boxes.map((box) => box.y + box.height))
  return { x: left, y: top, width: right - left, height: bottom - top }
}

export function centre(box: Box): Point {
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 }
}

export function midpoint(a: Point, b: Point): Point {
  return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }
}

/** Where the segment from the centre of `box` towards `toward` leaves the box. */
export function borderPoint(box: Box, toward: Point): Point {
  const origin = centre(box)
  const dx = toward.x - origin.x
  const dy = toward.y - origin.y
  if (dx === 0 && dy === 0) return origin
  const scale = Math.min(
    dx === 0 ? Number.POSITIVE_INFINITY : box.width / 2 / Math.abs(dx),
    dy === 0 ? Number.POSITIVE_INFINITY : box.height / 2 / Math.abs(dy)
  )
  return { x: origin.x + dx * scale, y: origin.y + dy * scale }
}

/** `distance` along the route from its first point: where an end's own label sits. */
export function pointAlong(points: readonly Point[], distance: number): Point | null {
  let remaining = distance
  for (let index = 0; index < points.length - 1; index += 1) {
    const from = points[index]!
    const to = points[index + 1]!
    const length = Math.hypot(to.x - from.x, to.y - from.y)
    if (remaining <= length && length > 0) {
      const t = remaining / length
      return { x: from.x + (to.x - from.x) * t, y: from.y + (to.y - from.y) * t }
    }
    remaining -= length
  }
  return points[points.length - 1] ?? null
}
