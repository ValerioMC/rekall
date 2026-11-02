import type { Box, Point } from '@/features/diagram/lib/viewport'
import { pointAlong, polylineLength } from '@/features/flow/lib/polyline'

const GATE_CLEARANCE = 24
const GATE_FRACTION_MAX = 0.35
const GATE_FRACTION_MIN = 0.1
const FALLBACK_FRACTIONS: readonly number[] = Array.from({ length: 17 }, (_, step) => 0.1 + step * 0.05).sort(
  (a, b) => Math.abs(a - GATE_FRACTION_MAX) - Math.abs(b - GATE_FRACTION_MAX)
)
const CHARACTER_WIDTH = 7
const BADGE_CHROME = 40
const PILL_CHROME = 18
const BADGE_HEIGHT = 22
const BREATHING_ROOM = 4

/** A route that wants a gate: its path and the text the gate carries. */
export interface GateRequest {
  readonly key: string
  readonly points: readonly Point[]
  readonly label: string
}

/**
 * Where a session gate sits on a route: a third of the way along it, pulled back toward the
 * source on a short route so it stays clear of the outcome pill that sits at the middle.
 */
export function gateAt(points: readonly Point[]): Point {
  const length = polylineLength(points)
  if (length === 0) return points[0] ?? { x: 0, y: 0 }
  const fraction = Math.min(GATE_FRACTION_MAX, Math.max(GATE_FRACTION_MIN, 0.5 - GATE_CLEARANCE / length))
  return pointAlong(points, fraction * length)
}

/** The room an outcome pill takes, so a gate can keep off it. */
export function pillBox(at: Point, outcome: string): Box {
  return centredBox(at, outcome.length * CHARACTER_WIDTH + PILL_CHROME, BADGE_HEIGHT)
}

/**
 * A gate for each request, on its own route and clear of `obstacles` (steps, outcome pills) and of
 * the gates placed before it. The first spot along the route that is free wins; when none is, the
 * default spot stays.
 */
export function placeGates(requests: readonly GateRequest[], obstacles: readonly Box[]): Map<string, Point> {
  const taken = [...obstacles]
  const placed = new Map<string, Point>()
  for (const request of requests) {
    const width = request.label.length * CHARACTER_WIDTH + BADGE_CHROME
    const spots = spotsAlong(request.points)
    const free = spots.find((spot) => taken.every((box) => !overlaps(centredBox(spot, width, BADGE_HEIGHT), box)))
    const spot = free ?? spots[0]!
    placed.set(request.key, spot)
    taken.push(centredBox(spot, width, BADGE_HEIGHT))
  }
  return placed
}

function spotsAlong(points: readonly Point[]): Point[] {
  const length = polylineLength(points)
  return [gateAt(points), ...FALLBACK_FRACTIONS.map((fraction) => pointAlong(points, fraction * length))]
}

function centredBox(at: Point, width: number, height: number): Box {
  return { x: at.x - width / 2, y: at.y - height / 2, width, height }
}

function overlaps(a: Box, b: Box): boolean {
  return (
    a.x < b.x + b.width + BREATHING_ROOM &&
    b.x < a.x + a.width + BREATHING_ROOM &&
    a.y < b.y + b.height + BREATHING_ROOM &&
    b.y < a.y + a.height + BREATHING_ROOM
  )
}
