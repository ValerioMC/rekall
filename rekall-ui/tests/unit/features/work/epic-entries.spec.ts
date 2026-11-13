import { describe, expect, it } from 'vitest'
import { epicEntries } from '@/features/work/lib/epic-entries'
import { workRows } from '@/features/work/lib/work-rows'
import { task } from './work-fixtures'
import type { TaskId } from '@/model/branded'

const noTokens = (): number => 0
const noTitle = (): string | null => null

function rowsOf(...tasks: ReturnType<typeof task>[]) {
  return workRows(tasks, [], noTokens)
}

describe('epicEntries', () => {
  it('gathers the tasks of an epic under it and lists them nowhere else', () => {
    const rows = rowsOf(
      task('RK-1', { kind: 'EPIC' }),
      task('RK-2', { epicId: 'RK-1' as TaskId }),
      task('RK-3'),
      task('RK-4', { epicId: 'RK-1' as TaskId })
    )

    const entries = epicEntries(rows, noTitle)

    expect(entries.map((entry) => `${entry.kind}:${entry.row.task.code}`)).toEqual(['task:RK-3', 'epic:RK-1'])
    const epic = entries.find((entry) => entry.kind === 'epic')
    expect(epic?.kind === 'epic' && epic.members.map((member) => member.task.code)).toEqual(['RK-4', 'RK-2'])
  })

  it('keeps a task whose epic is not listed at its place and names the epic', () => {
    const rows = rowsOf(task('RK-2', { epicId: 'RK-1' as TaskId }))

    const [entry] = epicEntries(rows, () => 'Checkout rework')

    expect(entry).toMatchObject({ kind: 'task', epicTitle: 'Checkout rework' })
  })

  it('shows an epic with no listed task as an empty one', () => {
    const [entry] = epicEntries(rowsOf(task('RK-1', { kind: 'EPIC' })), noTitle)

    expect(entry).toMatchObject({ kind: 'epic', members: [] })
  })
})
