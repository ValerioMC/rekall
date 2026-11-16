import { describe, expect, it } from 'vitest'
import { pageOf } from '@/features/work/lib/work-pagination'

const rows = Array.from({ length: 60 }, (_, index) => index + 1)

describe('pageOf', () => {
  it('lists the first page of a long list with the range it covers', () => {
    const page = pageOf(rows, 1, 25)

    expect(page.rows).toHaveLength(25)
    expect(page).toMatchObject({ page: 1, pageCount: 3, total: 60, from: 1, to: 25 })
  })

  it('ends on a shorter last page', () => {
    const page = pageOf(rows, 3, 25)

    expect(page.rows).toEqual(rows.slice(50))
    expect(page).toMatchObject({ from: 51, to: 60 })
  })

  it('holds a page that does not exist between the first and the last', () => {
    expect(pageOf(rows, 9, 25).page).toBe(3)
    expect(pageOf(rows, 0, 25).page).toBe(1)
  })

  it('is one empty page for an empty list', () => {
    expect(pageOf([], 1, 25)).toMatchObject({ rows: [], page: 1, pageCount: 1, from: 0, to: 0 })
  })
})
