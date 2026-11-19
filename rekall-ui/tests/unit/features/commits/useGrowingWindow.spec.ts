import { describe, expect, it } from 'vitest'
import { ref } from 'vue'
import { useGrowingWindow } from '@/features/commits/composables/useGrowingWindow'

describe('useGrowingWindow', () => {
  it('shows one batch first and a batch more on each request', () => {
    const total = ref(25)
    const window = useGrowingWindow(() => total.value, 10)

    expect(window.shown.value).toBe(10)
    expect(window.remaining.value).toBe(15)

    window.showMore()
    expect(window.shown.value).toBe(20)

    window.showMore()
    expect(window.shown.value).toBe(25)
    expect(window.remaining.value).toBe(0)
  })

  it('never shows more than the list holds', () => {
    const window = useGrowingWindow(() => 4, 10)

    expect(window.shown.value).toBe(4)
    expect(window.remaining.value).toBe(0)
  })

  it('opens the batch that holds a given item and leaves a shown one alone', () => {
    const window = useGrowingWindow(() => 50, 10)

    window.showThrough(3)
    expect(window.shown.value).toBe(10)

    window.showThrough(34)
    expect(window.shown.value).toBe(35)
  })

  it('goes back to the first batch on reset', () => {
    const window = useGrowingWindow(() => 50, 10)
    window.showThrough(34)

    window.reset()

    expect(window.shown.value).toBe(10)
  })

  it('keeps the batches already shown when the list grows', () => {
    const total = ref(12)
    const window = useGrowingWindow(() => total.value, 10)
    window.showMore()

    total.value = 40

    expect(window.shown.value).toBe(20)
  })
})
