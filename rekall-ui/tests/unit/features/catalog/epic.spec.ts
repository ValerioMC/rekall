import { describe, expect, it } from 'vitest'
import { epicChoices, epicProgress, matchesEpicQuery, tasksOfEpic } from '@/features/catalog/model/epic'
import { taskFixture } from '../../support/taskFixture'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

const EPIC = 'epic' as TaskId

function task(id: string, over: Partial<Task> = {}): Task {
  return { ...taskFixture(id as TaskId), ...over }
}

describe('the tasks of an epic', () => {
  it('lists only the tasks that name it, live work first, then by title', () => {
    const tasks = [
      task('b', { epicId: EPIC, title: 'B', status: 'DONE' }),
      task('c', { epicId: EPIC, title: 'C', status: 'IN_PROGRESS' }),
      task('a', { epicId: EPIC, title: 'A', status: 'IN_PROGRESS' }),
      task('x', { title: 'X' })
    ]

    expect(tasksOfEpic(EPIC, tasks).map((entry) => entry.id)).toEqual(['a', 'c', 'b'])
  })
})

describe('how far an epic is', () => {
  it('counts done tasks, steps and work that waits on a review', () => {
    const children = [
      task('a', { status: 'DONE', stepCount: 2, stepsDone: 2 }),
      task('b', { stepCount: 3, stepsDone: 1 }),
      task('c', { reviewActive: true, reviewState: 'CLAIMED' })
    ]

    expect(epicProgress(children, new Set(['b' as TaskId]))).toEqual({
      total: 3,
      done: 1,
      stepCount: 5,
      stepsDone: 3,
      awaitingReview: 2
    })
  })
})

describe('the epics a task can belong to', () => {
  it('offers the epics of the same project, never the task itself', () => {
    const tasks = [
      task('e1', { kind: 'EPIC', title: 'Zeta' }),
      task('e2', { kind: 'EPIC', title: 'Alpha' }),
      task('e3', { kind: 'EPIC', projectId: 'other' as ProjectId }),
      task('t1')
    ]

    expect(epicChoices('p1', tasks, 'e1' as TaskId).map((entry) => entry.id)).toEqual(['e2'])
  })
})

describe('finding an epic by what was typed', () => {
  const epic = task('e1', { kind: 'EPIC', title: 'Report builder', label: 'report-builder', anchor: 'task:report-builder' })

  it('matches every typed word against the title, label or anchor, ignoring case', () => {
    expect(matchesEpicQuery(epic, '  REPORT build ')).toBe(true)
    expect(matchesEpicQuery(epic, 'task:report')).toBe(true)
    expect(matchesEpicQuery(epic, '')).toBe(true)
  })

  it('refuses the epic when one word appears nowhere', () => {
    expect(matchesEpicQuery(epic, 'report export')).toBe(false)
  })
})
