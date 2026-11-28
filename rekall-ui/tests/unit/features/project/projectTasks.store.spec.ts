import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import { installStorage } from '../../support/memoryStorage'
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

  const many = (count: number) => Array.from({ length: count }, (_, index) => task(`PR-${index + 1}`))

  it('pages the view and returns to the first page when the order changes', async () => {
    const { console_, projectTasks } = seed()
    console_.tasks = many(60)
    projectTasks.view = 'all'
    await nextTick()
    projectTasks.goToPage(3)
    expect(projectTasks.paging).toMatchObject({ page: 3, from: 51, to: 60, total: 60 })

    projectTasks.sortBy('updated')
    await nextTick()

    expect(projectTasks.paging.page).toBe(1)
  })

  it('keeps the page and the search on coming back to the same project, not to another', async () => {
    const { console_, projectTasks } = seed()
    console_.tasks = many(60)
    projectTasks.view = 'all'
    projectTasks.focusProject('p1' as ProjectId)
    await nextTick()
    projectTasks.goToPage(2)

    projectTasks.focusProject(null)
    projectTasks.focusProject('p1' as ProjectId)
    await nextTick()
    expect(projectTasks.paging.page).toBe(2)

    projectTasks.query = 'PR'
    await nextTick()
    projectTasks.focusProject('p2' as ProjectId)
    expect(projectTasks.query).toBe('')
  })

  it('keeps the filters of each project apart', () => {
    const { console_, projectTasks } = seed()
    console_.tasks = [task('PR-1', { quick: true }), task('PR-2')]
    projectTasks.view = 'all'
    projectTasks.focusProject('p1' as ProjectId)

    projectTasks.toggleFilter('type', 'quick')
    expect(projectTasks.shown.map((row) => row.task.code)).toEqual(['PR-1'])

    projectTasks.focusProject('p2' as ProjectId)
    expect(projectTasks.filters.type).toEqual([])

    projectTasks.focusProject('p1' as ProjectId)
    expect(projectTasks.filters.type).toEqual(['quick'])
  })

  it('drops the status filter when the view changes and keeps the others', async () => {
    const { projectTasks } = seed()
    projectTasks.toggleFilter('status', 'review')
    projectTasks.toggleFilter('type', 'planned')

    projectTasks.view = 'all'
    await nextTick()

    expect(projectTasks.filters).toMatchObject({ status: [], type: ['planned'] })
  })

  it('counts what each filter value would list among the rows of the view and the other filters', () => {
    const { console_, projectTasks } = seed()
    console_.tasks = [task('PR-1', { quick: true, status: 'TODO' }), task('PR-2'), task('PR-3', { status: 'DONE' })]
    console_.steps = []
    projectTasks.view = 'open'
    projectTasks.toggleFilter('type', 'quick')

    expect(projectTasks.optionsOf('status').map((option) => [option.value, option.count])).toEqual([['todo', 1]])
  })

  it('clears the search and the filters together', () => {
    const { projectTasks } = seed()
    projectTasks.query = 'nothing like it'
    projectTasks.toggleFilter('type', 'quick')
    expect(projectTasks.isNarrowed).toBe(true)

    projectTasks.clearNarrowing()

    expect(projectTasks.isNarrowed).toBe(false)
  })

  it('previews the picked task of the page, else its first one', () => {
    const { console_, projectTasks } = seed()
    console_.tasks = many(3)
    projectTasks.view = 'all'
    expect(projectTasks.previewed?.task.code).toBe('PR-3')

    projectTasks.preview('PR-1' as TaskId)
    projectTasks.preview('PR-1' as TaskId)

    expect(projectTasks.previewed?.task.code).toBe('PR-1')
  })

  it('puts the preview beside the list only when it is wanted and the list is wide enough', () => {
    const { projectTasks } = seed()
    expect(projectTasks.previewsBeside).toBe(false)

    projectTasks.measure(1900)
    expect(projectTasks.previewsBeside).toBe(true)

    projectTasks.sidePreview = false
    expect(projectTasks.previewsBeside).toBe(false)
  })

  it('keeps the order, the page size, the columns and the filters in the browser', async () => {
    installStorage()
    const { projectTasks } = seed()
    projectTasks.focusProject('p1' as ProjectId)
    projectTasks.sortBy('title')
    projectTasks.pageSize = 100
    projectTasks.toggleColumn('tag')
    projectTasks.toggleFilter('type', 'planned')
    await nextTick()

    setActivePinia(createPinia())
    const again = useProjectTasksStore()
    again.focusProject('p1' as ProjectId)

    expect(again.sort).toEqual({ key: 'title', descending: false })
    expect(again.pageSize).toBe(100)
    expect([...again.hiddenColumns]).toEqual(['tag'])
  })
})
