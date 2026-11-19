import { describe, expect, it } from 'vitest'
import { columnMaxWidth } from '@/features/task/model/task-width'

describe('columnMaxWidth', () => {
  it('limits the column to the width chosen, and leaves full unlimited', () => {
    expect(columnMaxWidth('narrow')).toBe('760px')
    expect(columnMaxWidth('standard')).toBe('1040px')
    expect(columnMaxWidth('wide')).toBe('1360px')
    expect(columnMaxWidth('full')).toBe('none')
  })
})
