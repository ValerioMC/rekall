import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import WorkingSetTaskTabs from '@/features/workingset/components/WorkingSetTaskTabs.vue'
import type { StepTally, TaskActivity } from '@/features/workingset/model/task-activity'
import type { ProjectId, TaskId } from '@/model/branded'
import { project, task } from './fixtures'

const VEGA = project('vega', 'Vega', 'c1')
const TASKS = [task('t1', 'vega'), task('t2', 'vega')]

function tabs(
  currentTaskId: TaskId | null = null,
  activities: ReadonlyMap<TaskId, TaskActivity> = new Map(),
  tallies: ReadonlyMap<TaskId, StepTally> = new Map(),
  reviewCounts: ReadonlyMap<TaskId, number> = new Map()
) {
  return mount(WorkingSetTaskTabs, { props: { project: VEGA, tasks: TASKS, currentTaskId, activities, tallies, reviewCounts } })
}

describe('the task tab row', () => {
  it('puts All tasks first with the project\'s task count, then a tab per open task in order', () => {
    const row = tabs()

    expect(row.get('[data-testid="working-set-all-tasks"]').text()).toContain('All tasks')
    expect(row.findAll('[data-testid="working-set-task"]').map((tab) => tab.attributes('title'))).toEqual(['t1 Task t1, No activity', 't2 Task t2, No activity'])
  })

  it('marks the current task, or All tasks when none is', () => {
    const onTask = tabs('t2' as TaskId)
    const onList = tabs(null)

    expect(onTask.findAll('[data-testid="working-set-task"]').map((tab) => tab.attributes('aria-selected'))).toEqual(['false', 'true'])
    expect(onTask.get('[data-testid="working-set-all-tasks"]').attributes('aria-selected')).toBe('false')
    expect(onList.get('[data-testid="working-set-all-tasks"]').attributes('aria-selected')).toBe('true')
  })

  it('says in the tab what each task is doing, in its own mark and in the title', () => {
    const row = tabs(null, new Map<TaskId, TaskActivity>([['t1' as TaskId, 'WORKING'], ['t2' as TaskId, 'EXECUTED']]))

    expect(row.findAll('[data-testid="task-activity-mark"]').map((mark) => mark.attributes('data-activity'))).toEqual(['WORKING', 'EXECUTED'])
    expect(row.findAll('[data-testid="working-set-task"]').map((tab) => tab.attributes('title'))).toEqual([
      't1 Task t1, A session is working',
      't2 Task t2, Work was executed'
    ])
  })

  it('shows executed of total on a task with steps, and nothing on one without', () => {
    const row = tabs(null, new Map(), new Map<TaskId, StepTally>([['t1' as TaskId, { executed: 3, total: 7 }], ['t2' as TaskId, { executed: 0, total: 0 }]]))
    const tallies = row.findAll('[data-testid="working-set-task-tally"]')

    expect(tallies.map((tally) => tally.text())).toEqual(['3/7'])
    expect(tallies[0]?.attributes('title')).toBe('3 of 7 steps executed')
    expect(row.find('[data-testid="working-set-task-review"]').exists()).toBe(false)
    expect(tallies[0]?.attributes('data-waiting')).toBeUndefined()
    expect(tallies[0]?.classes()).not.toContain('bg-accent')
  })

  it('fills the counter in amber and says so while claimed steps wait for review', () => {
    const row = tabs(
      null,
      new Map(),
      new Map<TaskId, StepTally>([['t1' as TaskId, { executed: 3, total: 7 }]]),
      new Map<TaskId, number>([['t1' as TaskId, 2]])
    )
    const tally = row.get('[data-testid="working-set-task-tally"]')

    expect(tally.attributes('data-waiting')).toBe('true')
    expect(tally.classes()).toContain('bg-accent')
    expect(tally.text()).toBe('3/7')
    expect(tally.attributes('title')).toBe('3 of 7 steps executed, 2 waiting for your review')
    expect(tally.attributes('aria-label')).toBe('3 of 7 steps executed, 2 waiting for your review')
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
