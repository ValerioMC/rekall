import { describe, expect, it } from 'vitest'
import { taskTabItems } from '@/features/task/lib/task-tab-items'
import { tabOfPane, taskTabOf } from '@/features/task/model/task-tab'
import { asTaskId } from '@/model/branded'

const NO_COUNTS = { steps: 0, claimedSteps: 0, notes: 0, deviations: 0, commits: 0 }

describe('taskTabItems', () => {
  it('links each tab to the task page and lists them in the page order', () => {
    const items = taskTabItems(asTaskId('t-1'), NO_COUNTS)

    expect(items.map((item) => item.label)).toEqual(['Description', 'Plan', 'Changes', 'Wrapup', 'Notes', 'Time', 'Deviations'])
    expect(items[4]?.to).toBe('/task/t-1/notes')
    expect(items[2]?.to).toBe('/task/t-1/changes')
  })

  it('counts steps, logged commits and notes only when there are some', () => {
    const items = taskTabItems(asTaskId('t-1'), { steps: 7, claimedSteps: 0, notes: 3, deviations: 0, commits: 5 })

    expect(items.map((item) => item.count)).toEqual([undefined, 7, 5, undefined, 3, undefined, undefined])
  })

  it('turns the Plan count hot while a step waits for review', () => {
    const items = taskTabItems(asTaskId('t-1'), { steps: 4, claimedSteps: 2, notes: 0, deviations: 0, commits: 0 })

    expect(items[1]?.hot).toBe(true)
    expect(items[2]?.hot).toBeUndefined()
  })
})

describe('taskTabItems for an epic', () => {
  it('leaves out the plan and the changes, which an epic has none of', () => {
    const items = taskTabItems(asTaskId('t-1'), NO_COUNTS, true)

    expect(items.map((item) => item.label)).toEqual(['Description', 'Wrapup', 'Notes', 'Time', 'Deviations'])
  })
})

describe('tabOfPane', () => {
  it('names the tab that shows what a console pane showed', () => {
    expect(tabOfPane('wrapup')).toBe('wrapup')
    expect(tabOfPane('description')).toBe('description')
    expect(tabOfPane('deviation')).toBe('deviations')
    expect(tabOfPane('steps')).toBe('plan')
  })

  it('names none for a note or the terminal, which stay where the reader is', () => {
    expect(tabOfPane('note')).toBeNull()
    expect(tabOfPane('terminal')).toBeNull()
  })
})

describe('taskTabOf', () => {
  it('reads a tab from its path segment and falls back to the description', () => {
    expect(taskTabOf('wrapup')).toBe('wrapup')
    expect(taskTabOf('nonsense')).toBe('description')
    expect(taskTabOf(undefined)).toBe('description')
  })
})
