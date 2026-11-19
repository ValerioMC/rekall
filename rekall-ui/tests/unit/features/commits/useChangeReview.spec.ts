import { describe, expect, it } from 'vitest'
import { useChangeReview } from '@/features/commits/composables/useChangeReview'

describe('useChangeReview', () => {
  it('folds a file once it is marked viewed and unmarks it when opened again', () => {
    const review = useChangeReview()

    review.toggleViewed('a.ts')
    expect([review.isViewed('a.ts'), review.isShut('a.ts')]).toEqual([true, true])

    review.toggleShut('a.ts')
    expect([review.isViewed('a.ts'), review.isShut('a.ts')]).toEqual([false, false])
  })

  it('folds a file by hand without marking it viewed', () => {
    const review = useChangeReview()

    review.toggleShut('a.ts')

    expect([review.isViewed('a.ts'), review.isShut('a.ts')]).toEqual([false, true])
  })

  it('forgets every mark on reset', () => {
    const review = useChangeReview()
    review.toggleViewed('a.ts')
    review.toggleShut('b.ts')

    review.reset()

    expect(review.viewed.value.size).toBe(0)
    expect(review.isShut('b.ts')).toBe(false)
  })
})
