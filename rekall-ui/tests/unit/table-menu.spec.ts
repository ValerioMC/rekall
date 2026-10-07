import { describe, expect, it } from 'vitest'
import { isTableActionAvailable, tableOperationFor } from '@/common/live-preview/table-menu'

describe('isTableActionAvailable', () => {
  it('refuses a row above the header and deleting the header', () => {
    const header = { row: 0, column: 0 }

    expect(isTableActionAvailable('rowAbove', header, 2)).toBe(false)
    expect(isTableActionAvailable('deleteRow', header, 2)).toBe(false)
    expect(isTableActionAvailable('rowBelow', header, 2)).toBe(true)
  })

  it('offers both on a body row', () => {
    expect(isTableActionAvailable('rowAbove', { row: 1, column: 0 }, 2)).toBe(true)
    expect(isTableActionAvailable('deleteRow', { row: 1, column: 0 }, 2)).toBe(true)
  })

  it('keeps the last column', () => {
    expect(isTableActionAvailable('deleteColumn', { row: 1, column: 0 }, 1)).toBe(false)
    expect(isTableActionAvailable('deleteColumn', { row: 1, column: 0 }, 2)).toBe(true)
  })
})

describe('tableOperationFor', () => {
  it('turns the clicked row into body-row positions', () => {
    const second = { row: 2, column: 0 }

    expect(tableOperationFor('rowAbove', second)).toEqual({ kind: 'insertRow', row: 1 })
    expect(tableOperationFor('rowBelow', second)).toEqual({ kind: 'insertRow', row: 2 })
    expect(tableOperationFor('deleteRow', second)).toEqual({ kind: 'deleteRow', row: 1 })
  })

  it('inserts below the header as the first body row', () => {
    expect(tableOperationFor('rowBelow', { row: 0, column: 0 })).toEqual({ kind: 'insertRow', row: 0 })
  })

  it('inserts a column either side of the clicked one', () => {
    const cell = { row: 1, column: 1 }

    expect(tableOperationFor('columnLeft', cell)).toEqual({ kind: 'insertColumn', column: 1 })
    expect(tableOperationFor('columnRight', cell)).toEqual({ kind: 'insertColumn', column: 2 })
  })

  it('aligns the clicked column', () => {
    expect(tableOperationFor('alignCenter', { row: 1, column: 3 })).toEqual({
      kind: 'align',
      column: 3,
      alignment: 'center'
    })
  })
})
