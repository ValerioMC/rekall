const FOUR_DECIMALS_BELOW_USD = 0.1

/** `$0.0123` below ten cents, where the small digits still say something, and `$0.42` above. */
export function formatCost(usd: number): string {
  return `$${usd.toFixed(usd < FOUR_DECIMALS_BELOW_USD ? 4 : 2)}`
}
