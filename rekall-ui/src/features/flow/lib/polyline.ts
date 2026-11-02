import type { Point } from '@/features/diagram/lib/viewport'

function distance(from: Point, to: Point): number {
  return Math.hypot(to.x - from.x, to.y - from.y)
}

export function polylineLength(points: readonly Point[]): number {
  return points.slice(1).reduce((sum, point, index) => sum + distance(points[index]!, point), 0)
}

export function pointAlong(points: readonly Point[], travel: number): Point {
  let remaining = travel
  for (let index = 1; index < points.length; index += 1) {
    const from = points[index - 1]!
    const to = points[index]!
    const segment = distance(from, to)
    if (remaining <= segment && segment > 0) {
      const ratio = remaining / segment
      return { x: from.x + (to.x - from.x) * ratio, y: from.y + (to.y - from.y) * ratio }
    }
    remaining -= segment
  }
  return points[points.length - 1]!
}
