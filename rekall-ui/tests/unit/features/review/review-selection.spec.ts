import { describe, expect, it } from 'vitest'
import { keyAfterLeaving } from '@/features/review/lib/review-selection'
import { reviewItemLine } from '@/features/review/lib/review-item-line'
import type { ReviewItem } from '@/features/catalog/model/review'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

function item(key: string): ReviewItem {
  return { key, taskId: 't' as TaskId, stepId: null, taskTitle: key, stepTitle: null, anchor: 'task:T-1', claimedAt: null }
}

const queue = ['a', 'b', 'c', 'd'].map(item)

describe('keyAfterLeaving', () => {
  it('keeps the item while it still waits', () => {
    expect(keyAfterLeaving(queue, queue, 'b')).toBe('b')
  })

  it('moves to the next item in the order the queue had', () => {
    expect(keyAfterLeaving(queue, [queue[0]!, queue[2]!, queue[3]!], 'b')).toBe('c')
  })

  it('moves to the one before when the last item left', () => {
    expect(keyAfterLeaving(queue, [queue[0]!, queue[1]!, queue[2]!], 'd')).toBe('c')
  })

  it('is null once the queue is empty', () => {
    expect(keyAfterLeaving(queue, [], 'a')).toBeNull()
  })

  it('falls back to the first waiting item for a key the queue never held', () => {
    expect(keyAfterLeaving(queue, queue, 'zz')).toBe('a')
  })
})

describe('reviewItemLine', () => {
  const task = { companyName: 'Acme', projectTitle: 'Vega' } as Task

  it('names the company, the project and the piece that waits', () => {
    expect(reviewItemLine(task, 3)).toBe('Acme · Vega · step 3')
    expect(reviewItemLine(task, null)).toBe('Acme · Vega · whole task')
  })
})
