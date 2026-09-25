import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import TaskMark from '@/components/console/TaskMark.vue'
import { taskMark } from '@/model/task-mark'
import type { Task, TaskStatus, TaskStep, TaskStepState } from '@/model/catalog'
import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'

function task(overrides: Partial<Task> = {}): Task {
  return {
    id: 't1' as TaskId,
    label: 't1',
    title: 'Task',
    status: 'IN_PROGRESS',
    description: null,
    projectId: 'p1' as ProjectId,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'acme',
    projectRepoFolder: null,
    documentCount: 0,
    stepCount: 0,
    stepsDone: 0,
    draftStepCount: 0,
    hasWrapup: false,
    reviewState: 'OPEN',
    reviewActive: true,
    claimedAt: null,
    acceptedAt: null,
    reviewNote: null,
    tagId: null,
    tagName: null,
    tagIcon: null,
    tagColor: null,
    anchor: 'project:vega task:t1',
    updatedAt: '2026-09-20T10:00:00Z',
    ...overrides
  }
}

function step(id: string, state: TaskStepState, taskId = 't1'): TaskStep {
  return {
    id: id as TaskStepId,
    taskId: taskId as TaskId,
    title: `Step ${id}`,
    bodyMarkdown: null,
    state,
    done: state === 'DONE',
    runningAt: null,
    claimedAt: null,
    doneAt: null,
    position: 0,
    createdAt: '2026-09-20T09:00:00Z',
    updatedAt: '2026-09-20T09:00:00Z'
  }
}

/** A task with a checklist, its counts matching the steps given. */
function checklistTask(steps: readonly TaskStep[], status: TaskStatus = 'IN_PROGRESS'): Task {
  const checklist = steps.filter((candidate) => candidate.state !== 'DRAFT')
  return task({
    status,
    reviewActive: checklist.length === 0,
    stepCount: checklist.length,
    stepsDone: checklist.filter((candidate) => candidate.done).length
  })
}

describe('taskMark', () => {
  it('waits on an in-progress task nothing has been handed in for', () => {
    expect(taskMark(task(), [], false)).toEqual({ state: 'WAITING', accepted: 0 })
  })

  it('claims a stepless task whose wrapup claimed it, and accepts it once reviewed', () => {
    expect(taskMark(task({ reviewState: 'CLAIMED' }), [], false).state).toBe('CLAIMED')
    expect(taskMark(task({ reviewState: 'DONE' }), [], false).state).toBe('ACCEPTED')
  })

  it('claims a checklist with any claimed step, even when others are accepted', () => {
    const steps = [step('a', 'DONE'), step('b', 'CLAIMED'), step('c', 'OPEN')]

    const mark = taskMark(checklistTask(steps), steps, false)

    expect(mark.state).toBe('CLAIMED')
    expect(mark.accepted).toBeCloseTo(1 / 3)
  })

  it('accepts a checklist only when every step is done, ignoring drafts', () => {
    const steps = [step('a', 'DONE'), step('b', 'DONE'), step('c', 'DRAFT')]

    expect(taskMark(checklistTask(steps), steps, false)).toEqual({ state: 'ACCEPTED', accepted: 1 })
  })

  it('keeps a partly accepted checklist waiting, with the accepted share', () => {
    const steps = [step('a', 'DONE'), step('b', 'OPEN')]

    expect(taskMark(checklistTask(steps), steps, false)).toEqual({ state: 'WAITING', accepted: 0.5 })
  })

  it('reads only its own steps', () => {
    const own = step('a', 'OPEN')
    const steps = [own, step('x', 'CLAIMED', 't2')]

    expect(taskMark(checklistTask([own]), steps, false).state).toBe('WAITING')
  })

  it('goes live for a timer, a running stepless session or a running step, whatever the status', () => {
    const running = [step('a', 'RUNNING'), step('b', 'CLAIMED')]

    expect(taskMark(task({ status: 'TODO' }), [], true).state).toBe('LIVE')
    expect(taskMark(task({ reviewState: 'RUNNING' }), [], false).state).toBe('LIVE')
    expect(taskMark(checklistTask(running), running, false).state).toBe('LIVE')
  })

  it('tells a running timer with a claim waiting apart from a running timer alone', () => {
    const claimed = [step('a', 'CLAIMED'), step('b', 'OPEN')]

    expect(taskMark(task({ reviewState: 'CLAIMED' }), [], true).state).toBe('LIVE_CLAIMED')
    expect(taskMark(checklistTask(claimed), claimed, true).state).toBe('LIVE_CLAIMED')
    expect(taskMark(task(), [], true).state).toBe('LIVE')
    expect(taskMark(checklistTask([step('a', 'OPEN')]), [step('a', 'OPEN')], true).state).toBe('LIVE')
  })

  it('stays live while a session is still at work, whatever it already claimed and the timer', () => {
    const working = [step('a', 'CLAIMED'), step('b', 'RUNNING')]

    expect(taskMark(checklistTask(working), working, true).state).toBe('LIVE')
  })

  it('rests on any status but in progress, claimed or not', () => {
    for (const status of ['TODO', 'BLOCKED', 'DONE'] as const) {
      expect(taskMark(task({ status, reviewState: 'CLAIMED' }), [], false).state).toBe('RESTING')
    }
  })
})

describe('TaskMark', () => {
  afterEach(() => {
    vi.useRealTimers()
  })

  it('draws a plain status dot at rest, and no ring', () => {
    const wrapper = mount(TaskMark, { props: { state: 'RESTING', status: 'BLOCKED' } })

    expect(wrapper.get('[data-testid="task-mark"]').attributes('data-status')).toBe('BLOCKED')
    expect(wrapper.find('.mark-dot').exists()).toBe(true)
    expect(wrapper.find('.mark-ring').exists()).toBe(false)
  })

  it('gives each in-progress face its own parts', () => {
    const face = (state: 'WAITING' | 'LIVE' | 'LIVE_CLAIMED' | 'CLAIMED' | 'ACCEPTED') =>
      mount(TaskMark, { props: { state, status: 'IN_PROGRESS' } })

    expect(face('WAITING').find('.mark-core').exists()).toBe(true)
    expect(face('WAITING').find('.mark-check').exists()).toBe(false)
    expect(face('LIVE').find('.mark-comet').exists()).toBe(true)
    expect(face('LIVE').find('.mark-check').exists()).toBe(false)
    expect(face('LIVE_CLAIMED').find('.mark-comet').exists()).toBe(true)
    expect(face('LIVE_CLAIMED').find('.mark-check').exists()).toBe(true)
    expect(face('LIVE_CLAIMED').find('.mark-core').exists()).toBe(false)
    expect(face('CLAIMED').get('.mark-ring').attributes('transform')).toBe('rotate(25 7 7)')
    expect(face('CLAIMED').find('.mark-check').exists()).toBe(true)
    expect(face('ACCEPTED').find('.mark-engrave').exists()).toBe(true)
    expect(face('ACCEPTED').find('.mark-comet').exists()).toBe(false)
  })

  it('draws the accepted share as an arc only while waiting', () => {
    const waiting = mount(TaskMark, { props: { state: 'WAITING', status: 'IN_PROGRESS', accepted: 0.25 } })
    const none = mount(TaskMark, { props: { state: 'WAITING', status: 'IN_PROGRESS' } })

    expect(waiting.get('[data-testid="task-mark-progress"]').attributes('stroke-dasharray')).toBe('25.0 100')
    expect(none.find('[data-testid="task-mark-progress"]').exists()).toBe(false)
  })

  it('plays an arrival on a change of state, not on mount, and only once', async () => {
    vi.useFakeTimers()
    const wrapper = mount(TaskMark, { props: { state: 'CLAIMED', status: 'IN_PROGRESS' } })
    const svg = () => wrapper.get('[data-testid="task-mark"]')

    expect(svg().attributes('data-arrive')).toBeUndefined()

    await wrapper.setProps({ state: 'ACCEPTED' })
    expect(svg().attributes('data-arrive')).toBe('ACCEPTED')
    expect(wrapper.find('.mark-ripple').exists()).toBe(true)

    vi.advanceTimersByTime(900)
    await wrapper.vm.$nextTick()
    expect(svg().attributes('data-arrive')).toBeUndefined()
  })
})
