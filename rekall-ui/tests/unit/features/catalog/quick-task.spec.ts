import { describe, expect, it } from 'vitest'
import { activeQuickTask, withoutQuickTasks } from '@/features/catalog/lib/quick-task'
import { taskFixture } from '../../support/taskFixture'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

const PROJECT = 'p1' as ProjectId

function quick(id: string, change: Partial<Task> = {}): Task {
  return { ...taskFixture(id as TaskId), quick: true, ...change }
}

describe('activeQuickTask', () => {
  it('is a quick task of the project with a session on it', () => {
    expect(activeQuickTask([quick('a', { reviewState: 'RUNNING' })], PROJECT)?.id).toBe('a')
  })

  it('is none when the session ended without a wrapup and the task is back to open', () => {
    expect(activeQuickTask([quick('a', { reviewState: 'OPEN' })], PROJECT)).toBeNull()
  })

  it('is none once the wrapup claimed it, or it was accepted', () => {
    expect(activeQuickTask([quick('a', { reviewState: 'CLAIMED' })], PROJECT)).toBeNull()
    expect(activeQuickTask([quick('a', { reviewState: 'DONE' })], PROJECT)).toBeNull()
  })

  it('ignores plain tasks and quick tasks of another project', () => {
    const plain = taskFixture('b' as TaskId)
    const elsewhere = quick('c', { projectId: 'p2' as ProjectId })
    expect(activeQuickTask([plain, elsewhere], PROJECT)).toBeNull()
  })
})

describe('withoutQuickTasks', () => {
  const tasks = [quick('a'), taskFixture('b' as TaskId)]

  it('drops the quick tasks when asked to hide them', () => {
    expect(withoutQuickTasks(tasks, true).map((task) => task.id)).toEqual(['b'])
  })

  it('keeps every task otherwise', () => {
    expect(withoutQuickTasks(tasks, false)).toHaveLength(2)
  })
})
