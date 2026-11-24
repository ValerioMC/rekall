import { describe, expect, it } from 'vitest'
import { reviewKeyAction } from '@/features/review/lib/review-keys'

function press(key: string, modifiers: Partial<Pick<KeyboardEvent, 'metaKey' | 'ctrlKey' | 'altKey' | 'repeat'>> = {}) {
  return { key, metaKey: false, ctrlKey: false, altKey: false, repeat: false, ...modifiers }
}

describe('reviewKeyAction', () => {
  it('maps J and K to moving, A to accepting and S to sending back, in either case', () => {
    expect(reviewKeyAction(press('j'))).toBe('next')
    expect(reviewKeyAction(press('K'))).toBe('previous')
    expect(reviewKeyAction(press('a'))).toBe('accept')
    expect(reviewKeyAction(press('S'))).toBe('send-back')
  })

  it('leaves every other key alone', () => {
    expect(reviewKeyAction(press('x'))).toBeNull()
    expect(reviewKeyAction(press('Enter'))).toBeNull()
  })

  it('ignores a chord and a held key, so ⌘A and a repeating A never decide', () => {
    expect(reviewKeyAction(press('a', { metaKey: true }))).toBeNull()
    expect(reviewKeyAction(press('a', { ctrlKey: true }))).toBeNull()
    expect(reviewKeyAction(press('a', { repeat: true }))).toBeNull()
  })
})
