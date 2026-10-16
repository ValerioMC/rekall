import { describe, expect, it } from 'vitest'
import { compareText } from '@/features/catalog/lib/compare-text'

describe('compareText', () => {
  it('ignores case', () => {
    expect(['beta', 'Alpha', 'gamma'].sort(compareText)).toEqual(['Alpha', 'beta', 'gamma'])
  })

  it('orders numbers by value', () => {
    expect(['task 10', 'task 2', 'task 1'].sort(compareText)).toEqual(['task 1', 'task 2', 'task 10'])
  })
})
