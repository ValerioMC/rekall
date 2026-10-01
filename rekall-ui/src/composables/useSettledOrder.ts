import { computed, ref, watch, type ComputedRef, type WatchSource } from 'vue'

/**
 * Keeps a picker's rows where the reader first saw them. Ticking a row writes to the store, and
 * the list that comes back (attached-first, refetched, re-sorted) would otherwise jump under the
 * pointer. Each key keeps the place it was first given; keys never seen before join at the end.
 * A change of `resetOn` (a new filter, a different parent) lets the source order count again.
 */
export function useSettledOrder<Item>(
  source: () => readonly Item[],
  keyOf: (item: Item) => string,
  resetOn: WatchSource[] = []
): ComputedRef<Item[]> {
  const places = new Map<string, number>()
  const epoch = ref(0)

  if (resetOn.length) {
    watch(resetOn, () => {
      places.clear()
      epoch.value += 1
    })
  }

  return computed(() => {
    void epoch.value
    const items = source()
    for (const item of items) {
      const key = keyOf(item)
      if (!places.has(key)) places.set(key, places.size)
    }
    return [...items].sort((a, b) => places.get(keyOf(a))! - places.get(keyOf(b))!)
  })
}
