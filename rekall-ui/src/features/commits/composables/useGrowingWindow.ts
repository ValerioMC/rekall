import { computed, ref, type ComputedRef } from 'vue'

/** How many items of a long list are rendered: a first batch, then a batch more each time it is asked for. */
export function useGrowingWindow(
  total: () => number,
  batchSize: number
): {
  shown: ComputedRef<number>
  remaining: ComputedRef<number>
  showMore: () => void
  showThrough: (index: number) => void
  reset: () => void
} {
  const requested = ref(batchSize)
  const shown = computed(() => Math.min(requested.value, total()))
  const remaining = computed(() => total() - shown.value)

  function showMore(): void {
    requested.value = shown.value + batchSize
  }

  function showThrough(index: number): void {
    if (index >= shown.value) requested.value = index + 1
  }

  function reset(): void {
    requested.value = batchSize
  }

  return { shown, remaining, showMore, showThrough, reset }
}
