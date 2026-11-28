import { describe, expect, it } from 'vitest'
import { listedTasks } from '@/features/project/lib/listed-tasks'
import { taskRows } from '@/features/task/lib/task-rows'
import type { TimeEntry } from '@/features/time/model/time-entry'
import type { TaskId, TimeEntryId } from '@/model/branded'
import { step, task } from './project-task-fixtures'

const entry = (id: string, taskId: string, startedAt: string, stoppedAt: string | null): TimeEntry => ({
  id: id as TimeEntryId,
  taskId: taskId as TaskId,
  taskCode: taskId,
  taskTitle: taskId,
  projectLabel: 'vega',
  anchor: `task:${taskId}`,
  startedAt,
  stoppedAt,
  createdAt: startedAt,
  updatedAt: startedAt
})

describe('listedTasks', () => {
  it('gives each row its own step line and tracked time, a running timer counted up to now', () => {
    const rows = taskRows([task('RK-1', { stepCount: 1 }), task('RK-2')], [step('s1', 'RK-1', 'OPEN')], () => 0)
    const entries = [
      entry('e1', 'RK-1', '2026-09-20T10:00:00Z', '2026-09-20T10:01:00Z'),
      entry('e2', 'RK-1', '2026-09-20T11:59:00Z', null)
    ]

    const listed = listedTasks(rows, [step('s1', 'RK-1', 'OPEN')], entries, Date.parse('2026-09-20T12:00:00Z'))

    expect(listed.map((row) => [row.task.code, row.line.text, row.trackedSeconds])).toEqual([
      ['RK-2', '', 0],
      ['RK-1', 'Step s1', 120]
    ])
  })
})
