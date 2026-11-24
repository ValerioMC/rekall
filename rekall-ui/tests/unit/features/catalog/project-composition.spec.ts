import { describe, expect, it } from 'vitest'
import { projectComposition, reviewWaiting } from '@/features/catalog/lib/project-composition'
import type { TaskRowKind, TaskRow } from '@/features/task/model/task-row'
import { task } from '../project/project-task-fixtures'

function row(kind: TaskRowKind, needs = 0): TaskRow {
  return { task: task(`${kind}-${needs}`), kind, claimedSteps: needs, needs, running: kind === 'running', tokens: 0 }
}

describe('projectComposition', () => {
  it('sorts each task into finished, waiting, being worked or not started', () => {
    const rows = [row('filed'), row('filed'), row('review', 1), row('running'), row('progress'), row('blocked'), row('todo'), row('backlog')]

    expect(projectComposition(rows)).toEqual({ filed: 2, review: 1, active: 3, todo: 2 })
  })

  it('counts what waits for review the way the queue does, claims and tasks handed in whole', () => {
    expect(reviewWaiting([row('review', 2), row('progress', 1), row('todo')])).toBe(3)
  })
})
