import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useWorkStore } from '@/features/work/store/work.store'
import type { TaskId } from '@/model/branded'
import { project, step, task } from './work-fixtures'

function seed() {
  setActivePinia(createPinia())
  const console_ = useConsoleStore()
  console_.isLoading = false
  console_.projects = [project('p1', 'Vega')]
  console_.tasks = [task('PR-1', { stepCount: 1 }), task('PR-2')]
  console_.steps = [step('s1', 'PR-1', 'CLAIMED')]
  return { console_, work: useWorkStore() }
}

describe('the work store', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
    vi.useFakeTimers()
  })
  afterEach(() => vi.useRealTimers())

  it('keeps a task listed for a moment after a decision took it out of its tab', () => {
    const { console_, work } = seed()
    expect(work.shown.map((row) => row.task.code)).toEqual(['PR-1'])

    work.linger('PR-1' as TaskId)
    console_.steps = [step('s1', 'PR-1', 'DONE')]
    expect(work.shown.map((row) => row.task.code)).toEqual(['PR-1'])

    vi.advanceTimersByTime(1500)
    expect(work.shown).toEqual([])
  })

  it('keeps one peek open at a time and closes it on a second press', () => {
    const { work } = seed()

    work.togglePeek('PR-1' as TaskId)
    work.togglePeek('PR-2' as TaskId)
    expect(work.peekedTaskId).toBe('PR-2')

    work.togglePeek('PR-2' as TaskId)
    expect(work.peekedTaskId).toBeNull()
  })
})
