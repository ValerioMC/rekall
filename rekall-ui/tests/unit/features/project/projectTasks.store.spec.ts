import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import type { ProjectId, TaskId } from '@/model/branded'
import { project, step, task } from './project-task-fixtures'

function seed() {
  setActivePinia(createPinia())
  const console_ = useConsoleStore()
  console_.isLoading = false
  console_.projects = [project('p1', 'Vega')]
  console_.tasks = [task('PR-1', { stepCount: 1 }), task('PR-2')]
  console_.steps = [step('s1', 'PR-1', 'CLAIMED')]
  return { console_, projectTasks: useProjectTasksStore() }
}

describe('the project tasks store', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
    vi.useFakeTimers()
  })
  afterEach(() => vi.useRealTimers())

  it('keeps a task listed for a moment after a decision took it out of its tab', () => {
    const { console_, projectTasks } = seed()
    expect(projectTasks.shown.map((row) => row.task.code)).toEqual(['PR-1'])

    projectTasks.linger('PR-1' as TaskId)
    console_.steps = [step('s1', 'PR-1', 'DONE')]
    expect(projectTasks.shown.map((row) => row.task.code)).toEqual(['PR-1'])

    vi.advanceTimersByTime(1500)
    expect(projectTasks.shown).toEqual([])
  })

  it('keeps one peek open at a time and closes it on a second press', () => {
    const { projectTasks } = seed()

    projectTasks.togglePeek('PR-1' as TaskId)
    projectTasks.togglePeek('PR-2' as TaskId)
    expect(projectTasks.peekedTaskId).toBe('PR-2')

    projectTasks.togglePeek('PR-2' as TaskId)
    expect(projectTasks.peekedTaskId).toBeNull()
  })

  it('names the project in view: the one a page focuses, else the one the scope narrows to', () => {
    const { console_, projectTasks } = seed()
    expect(projectTasks.projectInView).toBeNull()

    console_.setScope(null, 'p1' as ProjectId)
    expect(projectTasks.projectInView).toBe('p1')

    projectTasks.focusProject('p2' as ProjectId)
    expect(projectTasks.projectInView).toBe('p2')
  })
})
