import { describe, expect, it } from 'vitest'
import { TASK_STATUS_COLOR, TASK_STATUSES } from '@/features/catalog/model/task-status'

describe('TASK_STATUS_COLOR', () => {
  it('gives every task status a colour', () => {
    for (const status of TASK_STATUSES) {
      expect(TASK_STATUS_COLOR[status]).toBeTruthy()
    }
  })

  it('never gives two statuses the same colour', () => {
    const colours = new Set(TASK_STATUSES.map((status) => TASK_STATUS_COLOR[status]))

    expect(colours.size).toBe(TASK_STATUSES.length)
  })
})
