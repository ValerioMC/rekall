import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import WorkingSetTaskTabs from '@/features/workingset/components/WorkingSetTaskTabs.vue'
import type { StepTally, TaskActivity } from '@/features/workingset/model/task-activity'
import type { SessionKind } from '@/features/claude/model/terminal'
import type { ProjectId, TaskId } from '@/model/branded'
import { project, task } from './fixtures'

const VEGA = project('vega', 'Vega', 'c1')
const TASKS = [task('t1', 'vega'), task('t2', 'vega')]

function tabs(
  currentTaskId: TaskId | null = null,
  activities: ReadonlyMap<TaskId, TaskActivity> = new Map(),
  tallies: ReadonlyMap<TaskId, StepTally> = new Map(),
  reviewCounts: ReadonlyMap<TaskId, number> = new Map(),
  session: { kinds?: ReadonlyMap<TaskId, SessionKind>; shown?: SessionKind } = {}
) {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { render: () => null } },
      { path: '/task/:id/:tab?', name: 'task', component: { render: () => null } }
    ]
  })
  return mount(WorkingSetTaskTabs, {
    global: { plugins: [router] },
    props: {
      project: VEGA,
      tasks: TASKS,
      currentTaskId,
      listShown: currentTaskId === null,
      activities,
      tallies,
      reviewCounts,
      sessionKinds: session.kinds ?? new Map<TaskId, SessionKind>(),
      shownKind: session.shown ?? null
    }
  })
}

function tabsWithSessionOnT1(kind: SessionKind, currentTaskId: TaskId | null, shown?: SessionKind) {
  return tabs(currentTaskId, new Map(), new Map(), new Map(), { kinds: new Map([['t1' as TaskId, kind]]), shown })
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

  it('links the executed-step count to Plan outside the task tab', () => {
    const row = tabs(null, new Map(), new Map<TaskId, StepTally>([['t1' as TaskId, { executed: 3, total: 7 }], ['t2' as TaskId, { executed: 0, total: 0 }]]))
    const tallies = row.findAll('[data-testid="working-set-task-tally"]')

    expect(tallies.map((tally) => tally.text())).toEqual(['3/7'])
    expect(tallies[0]?.attributes('href')).toBe('/task/t1/plan')
    expect(tallies[0]?.attributes('aria-label')).toBe('Open Plan for t1, 3 of 7 steps executed')
    expect(row.find('[role="tab"] [data-testid="working-set-task-tally"]').exists()).toBe(false)
    expect(tallies[0]?.attributes('data-waiting')).toBeUndefined()
    expect(tallies[0]?.classes()).toContain('key-quiet')
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
    expect(tally.classes()).toContain('key-gold')
    expect(tally.text()).toBe('3/7')
    expect(tally.attributes('title')).toBe('Open Plan for t1, 3 of 7 steps executed, 2 waiting for your review')
    expect(tally.attributes('aria-label')).toBe('Open Plan for t1, 3 of 7 steps executed, 2 waiting for your review')
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

  it('puts no key on a task with no session', () => {
    const row = tabs()

    expect(row.find('[data-testid="working-set-task-terminal"]').exists()).toBe(false)
    expect(row.find('[data-testid="working-set-task-chat"]').exists()).toBe(false)
  })

  describe.each([
    ['TERMINAL', 'working-set-task-terminal', 'Open the terminal of t1', 'CHAT'],
    ['CHAT', 'working-set-task-chat', 'Open the chat of t1', 'TERMINAL']
  ] as const)('the key of a %s session', (kind, testid, name, otherKind) => {
    const key = `[data-testid="${testid}"]`

    it('is the only key of its task, named for its session and outside the tab itself', () => {
      const row = tabsWithSessionOnT1(kind, null)

      expect(row.findAll('[aria-pressed]').map((button) => button.attributes('aria-label'))).toEqual([name])
      expect(row.find(`[role="tab"] ${key}`).exists()).toBe(false)
    })

    it('is pressed only while that view of that task is on screen', () => {
      const pressed = (row: ReturnType<typeof tabs>) => row.get(key).attributes('aria-pressed')

      expect(pressed(tabsWithSessionOnT1(kind, 't1' as TaskId, kind))).toBe('true')
      expect(pressed(tabsWithSessionOnT1(kind, 't1' as TaskId))).toBe('false')
      expect(pressed(tabsWithSessionOnT1(kind, 't2' as TaskId, kind))).toBe('false')
      expect(pressed(tabsWithSessionOnT1(kind, 't1' as TaskId, otherKind))).toBe('false')
    })

    it('asks for the session without selecting or closing the task', async () => {
      const row = tabsWithSessionOnT1(kind, null)

      await row.get(key).trigger('click')

      expect(row.emitted('openSession')).toEqual([['t1']])
      expect(row.emitted('select')).toBeUndefined()
      expect(row.emitted('close')).toBeUndefined()
    })
  })
})
