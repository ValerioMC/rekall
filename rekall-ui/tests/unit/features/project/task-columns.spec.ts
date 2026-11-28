import { describe, expect, it } from 'vitest'
import { columnAbsence, columnsFor, gridTemplate } from '@/features/project/lib/task-columns'
import type { OptionalTaskColumn } from '@/features/project/model/project-task-list'
import type { TagId } from '@/model/branded'
import { listed, task } from './project-task-fixtures'

const full = listed(
  [task('RK-1', { description: 'Do it', tagId: 'g1' as TagId, documentCount: 1, trackerIssueKey: '4' })],
  [],
  { 'RK-1': 10 }
).map((row) => ({ ...row, trackedSeconds: 60 }))

const ids = (width: number, hidden: OptionalTaskColumn[] = [], rows = full): string[] =>
  columnsFor({ width, hidden: new Set(hidden), rows }).map((column) => column.id)

describe('columnsFor', () => {
  it('draws only the columns every list needs when it is narrow', () => {
    expect(ids(900)).toEqual(['peek', 'mark', 'task', 'title', 'status', 'plan', 'updated', 'actions'])
  })

  it('adds columns as the list widens', () => {
    expect(ids(1100)).toContain('tokens')
    expect(ids(1220)).toContain('tag')
    expect(ids(1420)).toContain('step')
    expect(ids(1620)).toEqual(expect.arrayContaining(['notes', 'time']))
    expect(ids(1820)).toEqual(['peek', 'mark', 'task', 'title', 'step', 'tag', 'status', 'plan', 'delivery', 'notes', 'tokens', 'time', 'updated', 'actions'])
  })

  it('leaves out a column the reader took away', () => {
    expect(ids(2000, ['tag', 'time'])).not.toContain('tag')
    expect(ids(2000, ['tag', 'time'])).not.toContain('time')
  })

  it('leaves out a column no task of the project fills', () => {
    const bare = listed([task('RK-1')])

    expect(ids(2000, [], bare)).toEqual(['peek', 'mark', 'task', 'title', 'status', 'plan', 'updated', 'actions'])
  })
})

describe('columnAbsence', () => {
  it('says why a ticked column is not on screen', () => {
    expect(columnAbsence('delivery', { width: 1000, hidden: new Set(), rows: full })).toBe('needs more width')
    expect(columnAbsence('delivery', { width: 2000, hidden: new Set(), rows: listed([task('RK-1')]) })).toBe('nothing to show')
    expect(columnAbsence('delivery', { width: 2000, hidden: new Set(), rows: full })).toBeNull()
  })
})

describe('gridTemplate', () => {
  it('joins the tracks of the columns drawn', () => {
    expect(gridTemplate(columnsFor({ width: 900, hidden: new Set(), rows: full }))).toBe(
      '24px 16px 72px minmax(240px, 1fr) 136px 150px 66px 56px'
    )
  })
})
