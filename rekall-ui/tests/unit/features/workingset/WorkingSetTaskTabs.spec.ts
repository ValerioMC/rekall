import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import WorkingSetTaskTabs from '@/features/workingset/components/WorkingSetTaskTabs.vue'
import type { ProjectId, TaskId } from '@/model/branded'
import { project, task } from './fixtures'

const VEGA = project('vega', 'Vega', 'c1')
const TASKS = [task('t1', 'vega'), task('t2', 'vega')]

function tabs(currentTaskId: TaskId | null = null, reviewCounts: ReadonlyMap<TaskId, number> = new Map()) {
  return mount(WorkingSetTaskTabs, { props: { project: VEGA, tasks: TASKS, currentTaskId, reviewCounts } })
}

describe('the task tab row', () => {
  it('puts All tasks first with the project\'s task count, then a tab per open task in order', () => {
    const row = tabs()

    expect(row.get('[data-testid="working-set-all-tasks"]').text()).toContain('All tasks')
    expect(row.findAll('[data-testid="working-set-task"]').map((tab) => tab.attributes('title'))).toEqual(['t1 Task t1', 't2 Task t2'])
  })

  it('marks the current task, or All tasks when none is', () => {
    const onTask = tabs('t2' as TaskId)
    const onList = tabs(null)

    expect(onTask.findAll('[data-testid="working-set-task"]').map((tab) => tab.attributes('aria-selected'))).toEqual(['false', 'true'])
    expect(onTask.get('[data-testid="working-set-all-tasks"]').attributes('aria-selected')).toBe('false')
    expect(onList.get('[data-testid="working-set-all-tasks"]').attributes('aria-selected')).toBe('true')
  })

  it('shows the review count of a task with work waiting only', () => {
    const row = tabs(null, new Map([['t2' as TaskId, 3]]))

    expect(row.findAll('[data-testid="working-set-task-review"]').map((pill) => pill.text())).toEqual(['3'])
  })

  it('selects a task or the task list when a tab is pressed', async () => {
    const row = tabs()

    await row.findAll('[data-testid="working-set-task"]')[1]?.trigger('click')
    await row.get('[data-testid="working-set-all-tasks"]').trigger('click')
    await row.get('[data-testid="working-set-add"]').trigger('click')

    expect(row.emitted('select')).toEqual([
      [{ projectId: 'vega' as ProjectId, taskId: 't2' }],
      [{ projectId: 'vega' as ProjectId, taskId: null }],
      [{ projectId: 'vega' as ProjectId, taskId: null }]
    ])
  })

  it('closes a task with its cross and selects nothing', async () => {
    const row = tabs()

    await row.findAll('[data-testid="working-set-task-close"]')[0]?.trigger('click')

    expect(row.emitted('close')).toEqual([['t1']])
    expect(row.emitted('select')).toBeUndefined()
  })
})
