import type { ReviewItem } from '@/features/catalog/model/review'

/**
 * The item to show after `key` left the queue: the next one in the order it had, else the one
 * before it. `null` when the queue is empty. A key still waiting stays where it is.
 */
export function keyAfterLeaving(before: readonly ReviewItem[], now: readonly ReviewItem[], key: string): string | null {
  if (now.some((item) => item.key === key)) return key
  const at = before.findIndex((item) => item.key === key)
  const order = at === -1 ? [] : [...before.slice(at + 1), ...before.slice(0, at).reverse()]
  const waiting = new Set(now.map((item) => item.key))
  return order.find((item) => waiting.has(item.key))?.key ?? now[0]?.key ?? null
}
