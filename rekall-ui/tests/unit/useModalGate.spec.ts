import { describe, expect, it } from 'vitest'
import { useModalGate } from '@/composables/useModalGate'

/**
 * The shared flag `App.vue`'s global keyboard shortcuts check before acting on any key. A
 * counter rather than a boolean, so two overlapping consumers don't have the second one's
 * `close()` re-open the console's shortcuts while the first is still up.
 */
describe('useModalGate', () => {
  it('starts closed', () => {
    const { isModalOpen } = useModalGate()
    expect(isModalOpen.value).toBe(0)
  })

  it('is open while at least one consumer holds it open', () => {
    const a = useModalGate()
    a.open()
    expect(a.isModalOpen.value).toBeGreaterThan(0)
    a.close()
    expect(a.isModalOpen.value).toBe(0)
  })

  it('stays open until every opener has closed, and never goes negative', () => {
    const a = useModalGate()
    const b = useModalGate()

    a.open()
    b.open()
    expect(a.isModalOpen.value).toBe(2)

    a.close()
    expect(a.isModalOpen.value).toBe(1)

    b.close()
    expect(a.isModalOpen.value).toBe(0)

    b.close()
    expect(a.isModalOpen.value).toBe(0)
  })
})
