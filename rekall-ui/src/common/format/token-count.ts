import type { TokenTotals } from '@/features/tokens/model/token-usage'

const UNITS: readonly [number, string][] = [
  [1_000_000_000, 'B'],
  [1_000_000, 'M'],
  [1_000, 'k']
]

/** `950`, `12.4k`, `3.1M`: three significant figures at most, so a column of them stays narrow. */
export function formatTokens(count: number): string {
  const tokens = Math.max(0, Math.round(count))
  for (const [size, suffix] of UNITS) {
    if (tokens >= size) {
      const scaled = tokens / size
      const digits = scaled >= 100 ? 0 : scaled >= 10 ? 1 : 2
      return `${Number(scaled.toFixed(digits))}${suffix}`
    }
  }
  return String(tokens)
}

/** The tooltip under a count: what was generated, then what was read to generate it. */
export function describeTokens(totals: TokenTotals): string {
  const format = (count: number): string => count.toLocaleString('en-US')
  return [
    `${format(totals.generated)} tokens generated`,
    `${format(totals.input)} input · ${format(totals.cacheWrite)} cache write · ${format(totals.cacheRead)} cache read`
  ].join('\n')
}
