import { describe, expect, it } from 'vitest'
import { reviewCountsByTask } from '@/features/workingset/lib/review-counts'
import type { ReviewItem } from '@/features/catalog/model/review'
import type { TaskId } from '@/model/branded'

function item(key: string, taskId: string): ReviewItem {
  return { key, taskId: taskId as TaskId, stepId: null, taskTitle: 'T', stepTitle: null, anchor: 'a', claimedAt: null }
}

describe('review counts by task', () => {
  it('counts the pieces waiting on each task and leaves a task with none out', () => {
    const counts = reviewCountsByTask([item('a', 't1'), item('b', 't1'), item('c', 't2')])

    expect(counts.get('t1' as TaskId)).toBe(2)
    expect(counts.get('t2' as TaskId)).toBe(1)
    expect(counts.has('t3' as TaskId)).toBe(false)
  })
})
