import { describe, expect, it } from 'vitest'
import { folderTail, withoutQuickTasks } from '@/features/catalog/lib/quick-task'
import { taskFixture } from '../../support/taskFixture'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

function quick(id: string, change: Partial<Task> = {}): Task {
  return { ...taskFixture(id as TaskId), quick: true, ...change }
}

describe('withoutQuickTasks', () => {
  const tasks = [quick('a'), taskFixture('b' as TaskId)]

  it('drops the quick tasks when asked to hide them', () => {
    expect(withoutQuickTasks(tasks, true).map((task) => task.id)).toEqual(['b'])
  })

  it('keeps every task otherwise', () => {
    expect(withoutQuickTasks(tasks, false)).toHaveLength(2)
  })
})

describe('folderTail', () => {
  it('keeps the last two folders of a long path, where the project is named', () => {
    expect(folderTail('/Users/ada/Projects/Personal/rekall-rust')).toBe('…/Personal/rekall-rust')
  })

  it('keeps a short path whole', () => {
    expect(folderTail('/code/vega')).toBe('/code/vega')
  })
})
