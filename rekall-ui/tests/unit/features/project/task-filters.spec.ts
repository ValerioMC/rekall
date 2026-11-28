import { describe, expect, it } from 'vitest'
import { applicableFilters, filterOptions, hasFilters, passesFilters } from '@/features/project/lib/task-filters'
import { NO_TASK_FILTERS } from '@/features/project/model/project-task-list'
import type { TagId, TaskId } from '@/model/branded'
import { listed, task } from './project-task-fixtures'

const rows = listed([
  task('RK-1', { kind: 'EPIC', title: 'Webhooks' }),
  task('RK-2', { epicId: 'RK-1' as TaskId, tagId: 'g1' as TagId, tagName: 'ui' }),
  task('RK-3', { quick: true, status: 'BLOCKED', tagId: 'g2' as TagId, tagName: 'backend' }),
  task('RK-4', { status: 'TODO' })
])

const passing = (filters: Partial<typeof NO_TASK_FILTERS>): string[] =>
  rows.filter((row) => passesFilters(row, { ...NO_TASK_FILTERS, ...filters })).map((row) => row.task.code)

describe('passesFilters', () => {
  it('lets everything through with nothing picked', () => {
    expect(passing({})).toHaveLength(4)
  })

  it('takes any of the values picked in one filter', () => {
    expect(passing({ status: ['blocked', 'todo'] })).toEqual(['RK-4', 'RK-3'])
  })

  it('asks every filter at once', () => {
    expect(passing({ tag: ['g2'], type: ['planned'] })).toEqual([])
  })

  it('lists an epic with its tasks when the epic is picked', () => {
    expect(passing({ epic: ['RK-1'] })).toEqual(['RK-2', 'RK-1'])
  })

  it('finds the tasks with no tag and the tasks by type', () => {
    expect(passing({ tag: ['none'] })).toEqual(['RK-4', 'RK-1'])
    expect(passing({ type: ['quick'] })).toEqual(['RK-3'])
    expect(passing({ type: ['epic'] })).toEqual(['RK-1'])
  })

  it('can leave one filter out', () => {
    const filters = { ...NO_TASK_FILTERS, status: ['todo'] }

    expect(rows.filter((row) => passesFilters(row, filters, 'status'))).toHaveLength(4)
  })
})

describe('hasFilters', () => {
  it('says whether anything is picked', () => {
    expect(hasFilters(NO_TASK_FILTERS)).toBe(false)
    expect(hasFilters({ ...NO_TASK_FILTERS, epic: ['none'] })).toBe(true)
  })
})

describe('applicableFilters', () => {
  it('drops a tag or an epic the project no longer has and keeps statuses and types', () => {
    const stored = { status: ['review'], tag: ['g1', 'gone'], epic: ['gone'], type: ['quick'] }

    expect(applicableFilters(stored, rows)).toEqual({ status: ['review'], tag: ['g1'], epic: [], type: ['quick'] })
  })
})

describe('filterOptions', () => {
  it('counts each value and lists the missing one last', () => {
    expect(filterOptions('tag', rows, [], rows)).toEqual([
      { value: 'g2', label: 'backend', count: 1 },
      { value: 'g1', label: 'ui', count: 1 },
      { value: 'none', label: 'No tag', count: 2 }
    ])
  })

  it('names an epic by its title', () => {
    expect(filterOptions('epic', rows, [], rows).map((option) => option.label)).toEqual(['Webhooks', 'No epic'])
  })

  it('keeps statuses in the order work moves and a picked one even at zero', () => {
    const options = filterOptions('status', rows, ['review'], rows)

    expect(options.map((option) => `${option.label} ${option.count}`)).toEqual([
      'Awaiting review 0',
      'In progress 2',
      'Blocked 1',
      'To do 1'
    ])
  })
})
