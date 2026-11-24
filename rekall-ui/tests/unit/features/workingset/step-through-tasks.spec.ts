import { describe, expect, it } from 'vitest'
import { stepThroughTasks } from '@/features/workingset/lib/step-through-tasks'
import type { TaskId } from '@/model/branded'

const ORDER = ['a', 'b', 'c'] as TaskId[]

describe('stepping through the open tasks', () => {
  it('goes to the next and the previous task', () => {
    expect(stepThroughTasks(ORDER, 'a' as TaskId, 1)).toBe('b')
    expect(stepThroughTasks(ORDER, 'c' as TaskId, -1)).toBe('b')
  })

  it('wraps at the ends', () => {
    expect(stepThroughTasks(ORDER, 'c' as TaskId, 1)).toBe('a')
    expect(stepThroughTasks(ORDER, 'a' as TaskId, -1)).toBe('c')
  })

  it('goes to the first task forward and the last backward when none is in view', () => {
    expect(stepThroughTasks(ORDER, null, 1)).toBe('a')
    expect(stepThroughTasks(ORDER, null, -1)).toBe('c')
  })

  it('has nowhere to go when nothing is open', () => {
    expect(stepThroughTasks([], null, 1)).toBeNull()
  })
})
