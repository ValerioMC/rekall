export const IDENTITY_HUE_COUNT = 6

export interface IdentityHue {
  readonly index: number
  readonly base: string
  readonly soft: string
  readonly line: string
}

function hashString(value: string): number {
  let h = 2166136261
  for (let i = 0; i < value.length; i++) {
    h ^= value.charCodeAt(i)
    h = Math.imul(h, 16777619)
  }
  return h >>> 0
}

export function identityIndex(id: string): number {
  return hashString(id) % IDENTITY_HUE_COUNT
}

export function identityHue(id: string): IdentityHue {
  const index = identityIndex(id)
  const n = index + 1
  return {
    index,
    base: `var(--color-identity-${n})`,
    soft: `var(--color-identity-${n}-soft)`,
    line: `var(--color-identity-${n}-line)`
  }
}

function mulberry32(seed: number): () => number {
  let a = seed
  return () => {
    a |= 0
    a = (a + 0x6d2b79f5) | 0
    let t = Math.imul(a ^ (a >>> 15), 1 | a)
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

export interface TracePathOptions {
  readonly width: number
  readonly height: number
  readonly points?: number
}

export function tracePath(
  seed: string,
  series: readonly number[] | undefined,
  opts: TracePathOptions
): string {
  const { width, height } = opts
  const points = opts.points ?? 14
  const padding = 1.5
  const usableHeight = height - padding * 2

  const values = normalizeSeries(seed, series, points)
  const coords = values.map((value, i) => ({
    x: (i / (points - 1)) * width,
    y: padding + (1 - value) * usableHeight
  }))

  let d = `M ${coords[0]!.x.toFixed(2)} ${coords[0]!.y.toFixed(2)}`
  for (let i = 1; i < coords.length; i++) {
    const prev = coords[i - 1]!
    const curr = coords[i]!
    const midX = (prev.x + curr.x) / 2
    const midY = (prev.y + curr.y) / 2
    d += ` Q ${prev.x.toFixed(2)} ${prev.y.toFixed(2)} ${midX.toFixed(2)} ${midY.toFixed(2)}`
  }
  const last = coords[coords.length - 1]!
  d += ` L ${last.x.toFixed(2)} ${last.y.toFixed(2)}`
  return d
}

function normalizeSeries(
  seed: string,
  series: readonly number[] | undefined,
  points: number
): number[] {
  if (series && series.length >= 2) {
    const tail = series.slice(-points)
    const padded =
      tail.length < points
        ? [...Array(points - tail.length).fill(tail[0] ?? 0), ...tail]
        : tail
    const max = Math.max(...padded)
    const min = Math.min(...padded)
    if (max === min) return padded.map(() => 0.5)
    return padded.map((v) => (v - min) / (max - min))
  }

  const rng = mulberry32(hashString(seed))
  const values: number[] = [0.5]
  for (let i = 1; i < points; i++) {
    const next = values[i - 1]! + (rng() - 0.5) * 0.5
    values.push(Math.min(0.92, Math.max(0.08, next)))
  }
  return values
}

export function traceAreaPath(strokePath: string, width: number, height: number): string {
  return `${strokePath} L ${width} ${height} L 0 ${height} Z`
}
