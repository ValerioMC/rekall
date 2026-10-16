import { describe, expect, it } from 'vitest'
import { canMove, movedOne } from '@/features/project/lib/section-order'
import { asProjectSectionId } from '@/model/branded'

const a = asProjectSectionId('a')
const b = asProjectSectionId('b')
const c = asProjectSectionId('c')

describe('section order', () => {
  it('moves a section one place earlier or later', () => {
    expect(movedOne([a, b, c], c, 'up')).toEqual([a, c, b])
    expect(movedOne([a, b, c], a, 'down')).toEqual([b, a, c])
  })

  it('leaves the list alone at either end or for an unknown id', () => {
    expect(movedOne([a, b, c], a, 'up')).toEqual([a, b, c])
    expect(movedOne([a, b, c], c, 'down')).toEqual([a, b, c])
    expect(movedOne([a, b], asProjectSectionId('x'), 'up')).toEqual([a, b])
  })

  it('says which way a section can still move', () => {
    expect([canMove([a, b, c], a, 'up'), canMove([a, b, c], a, 'down')]).toEqual([false, true])
    expect([canMove([a, b, c], c, 'up'), canMove([a, b, c], c, 'down')]).toEqual([true, false])
    expect(canMove([a], a, 'down')).toBe(false)
    expect(canMove([a], asProjectSectionId('x'), 'down')).toBe(false)
  })
})
