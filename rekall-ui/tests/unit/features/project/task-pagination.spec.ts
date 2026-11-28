import { describe, expect, it } from 'vitest'
import { pageNumbers, pageOf } from '@/features/project/lib/task-pagination'

const rows = Array.from({ length: 60 }, (_, index) => index + 1)

describe('pageOf', () => {
  it('lists the first page of a long list with the range it covers', () => {
    const page = pageOf(rows, 1, 25)

    expect(page.items).toHaveLength(25)
    expect(page).toMatchObject({ page: 1, pageCount: 3, total: 60, from: 1, to: 25 })
  })

  it('ends on a shorter last page', () => {
    const page = pageOf(rows, 3, 25)

    expect(page.items).toEqual(rows.slice(50))
    expect(page).toMatchObject({ from: 51, to: 60 })
  })

  it('holds a page that does not exist between the first and the last', () => {
    expect(pageOf(rows, 9, 25).page).toBe(3)
    expect(pageOf(rows, 0, 25).page).toBe(1)
  })

  it('is one empty page for an empty list', () => {
    expect(pageOf([], 1, 25)).toMatchObject({ items: [], page: 1, pageCount: 1, total: 0, from: 0, to: 0 })
  })

  it('moves an item that does not fit to the next page whole', () => {
    const weights = [3, 3, 1]
    const first = pageOf(weights, 1, 5, (weight) => weight)
    const second = pageOf(weights, 2, 5, (weight) => weight)

    expect(first).toMatchObject({ items: [3], pageCount: 2, total: 7, from: 1, to: 3 })
    expect(second).toMatchObject({ items: [3, 1], from: 4, to: 7 })
  })

  it('gives an item heavier than a page a page to itself', () => {
    expect(pageOf([9, 1], 1, 5, (weight) => weight)).toMatchObject({ items: [9], pageCount: 2, to: 9 })
  })
})

describe('pageNumbers', () => {
  it('offers every page of a short list', () => {
    expect(pageNumbers(2, 3)).toEqual([1, 2, 3])
  })

  it('offers the ends and the neighbours, with a gap between', () => {
    expect(pageNumbers(5, 9)).toEqual([1, null, 4, 5, 6, null, 9])
  })
})
