import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import QueueOrderRow from '@/features/queue/components/QueueOrderRow.vue'
import QueueStatusLine from '@/features/queue/components/QueueStatusLine.vue'
import QueueStepList from '@/features/queue/components/QueueStepList.vue'
import {
  EMPTY_RUN_QUEUE,
  asRunQueueItemId,
  runProgress,
  type RunQueueItem,
  type RunQueueItemState,
  type RunQueueStep,
  type RunQueueStepState
} from '@/features/queue/model/runQueue'
import { asTaskId } from '@/model/branded'

function step(title: string, state: RunQueueStepState, current = false): RunQueueStep {
  return { title, state, current }
}

function itemWith(state: RunQueueItemState, steps: RunQueueStep[], detail: string | null = null): RunQueueItem {
  return {
    id: asRunQueueItemId('a'),
    taskId: asTaskId('t-a'),
    taskTitle: 'Report a',
    taskLabel: 't-a',
    projectLabel: 'vega',
    anchor: 'project:vega task:t-a',
    position: 0,
    state,
    detail,
    startedAt: null,
    finishedAt: null,
    steps
  }
}

function mountRow(item: RunQueueItem) {
  return mount(QueueOrderRow, {
    props: { item, position: 1, last: false, live: true, busy: false, canMoveEarlier: false, canMoveLater: false }
  })
}

const SIX_STEPS = [
  step('One', 'CLAIMED'),
  step('Two', 'CLAIMED'),
  step('Three', 'RUNNING', true),
  step('Four', 'OPEN'),
  step('Five', 'OPEN'),
  step('Six', 'OPEN')
]

describe('the steps listed under a queued task', () => {
  it('lists each step with the mark of its state, titles in full on hover', () => {
    const list = mount(QueueStepList, { props: { steps: SIX_STEPS, live: true, listId: 'steps' } })

    const rows = list.findAll('[data-testid="queue-step"]')
    expect(rows).toHaveLength(6)
    expect(rows.map((row) => row.get('[data-testid="queue-bead"]').attributes('data-state'))).toEqual([
      'FINISHED',
      'FINISHED',
      'RUNNING',
      'QUEUED',
      'QUEUED',
      'QUEUED'
    ])
    expect(rows[0]!.find('span[title]').attributes('title')).toBe('One')
  })

  it('marks the current step beyond colour: aria-current, bolder, and the word for a screen reader', () => {
    const list = mount(QueueStepList, { props: { steps: SIX_STEPS, live: true, listId: 'steps' } })

    const current = list.findAll('[data-testid="queue-step"]').filter((row) => row.attributes('data-current') === 'true')
    expect(current).toHaveLength(1)
    expect(current[0]!.attributes('aria-current')).toBe('step')
    expect(current[0]!.classes()).toContain('font-medium')
    expect(current[0]!.get('.sr-only').text()).toBe('Current. Running')
  })

  it('wears the same mark for claimed and accepted but names them apart', () => {
    const list = mount(QueueStepList, {
      props: { steps: [step('Claimed one', 'CLAIMED'), step('Accepted one', 'DONE')], live: false, listId: 'steps' }
    })

    const rows = list.findAll('[data-testid="queue-step"]')
    expect(rows.map((row) => row.get('[data-testid="queue-bead"]').attributes('data-state'))).toEqual(['FINISHED', 'FINISHED'])
    expect(rows[0]!.get('.sr-only').text()).toBe('Claimed, waiting for your review')
    expect(rows[1]!.get('.sr-only').text()).toBe('Accepted')
  })

  it('draws a not-needed step as a dashed ring with its title muted and the words beside it', () => {
    const list = mount(QueueStepList, {
      props: { steps: [step('Skipped', 'NOT_NEEDED')], live: false, listId: 'steps' }
    })

    const row = list.get('[data-testid="queue-step"]')
    expect(row.get('[data-testid="queue-bead"]').attributes('data-state')).toBe('NOT_NEEDED')
    expect(row.get('[data-testid="queue-bead"]').classes()).toContain('border-dashed')
    expect(row.text()).toContain('Not needed')
    expect(row.find('span[title]').classes()).toContain('text-text-muted')
  })

  it('offers nothing to press', () => {
    const list = mount(QueueStepList, { props: { steps: SIX_STEPS, live: true, listId: 'steps' } })

    expect(list.find('button').exists()).toBe(false)
  })
})

describe('a queue row with steps', () => {
  it('shows nothing new for a task with no checklist', () => {
    const row = mountRow(itemWith('QUEUED', []))

    expect(row.find('[data-testid="queue-steps-toggle"]').exists()).toBe(false)
    expect(row.find('[data-testid="queue-step-list"]').exists()).toBe(false)
  })

  it('counts the steps that count, and opens the running task\'s list by default', () => {
    const row = mountRow(itemWith('RUNNING', [...SIX_STEPS, step('Unrun', 'NOT_NEEDED')]))

    const toggle = row.get('[data-testid="queue-steps-toggle"]')
    expect(toggle.text()).toBe('2 of 6 steps')
    expect(toggle.attributes('aria-expanded')).toBe('true')
    expect(toggle.attributes('aria-controls')).toBe(row.get('[data-testid="queue-step-list"]').attributes('id'))
    expect(row.findAll('[data-testid="queue-step"]')).toHaveLength(7)
  })

  it('keeps every other task\'s list collapsed until it is opened, and closes it again', async () => {
    const row = mountRow(itemWith('QUEUED', SIX_STEPS.map((s) => step(s.title, 'OPEN'))))
    const toggle = row.get('[data-testid="queue-steps-toggle"]')

    expect(toggle.attributes('aria-expanded')).toBe('false')
    expect(row.find('[data-testid="queue-step-list"]').exists()).toBe(false)

    await toggle.trigger('click')
    expect(toggle.attributes('aria-expanded')).toBe('true')
    expect(row.find('[data-testid="queue-step-list"]').exists()).toBe(true)

    await toggle.trigger('click')
    expect(row.find('[data-testid="queue-step-list"]').exists()).toBe(false)
  })

  it('opens the list of a task that starts running, unless the person closed it', async () => {
    const waiting = itemWith('QUEUED', SIX_STEPS)
    const row = mountRow(waiting)
    expect(row.find('[data-testid="queue-step-list"]').exists()).toBe(false)

    await row.setProps({ item: { ...waiting, state: 'RUNNING' } })

    expect(row.find('[data-testid="queue-step-list"]').exists()).toBe(true)
  })

  it('changes nothing about the task row\'s moves', () => {
    const row = mountRow(itemWith('QUEUED', SIX_STEPS))

    expect(row.find('[data-testid="run-queue-remove"]').exists()).toBe(true)
  })

  it('says a finished task\'s claimed steps wait for the person\'s review', () => {
    const row = mountRow(itemWith('FINISHED', [step('One', 'CLAIMED'), step('Two', 'CLAIMED')]))

    expect(row.text()).toContain('Claimed, waiting for your review')
  })

  it('says a stepless finished task is waiting for review too', () => {
    expect(mountRow(itemWith('FINISHED', [])).text()).toContain('Claimed, waiting for your review')
  })

  it('says a flow that ended early left steps it did not need', () => {
    const row = mountRow(
      itemWith('FINISHED', [
        step('One', 'CLAIMED'),
        step('A', 'NOT_NEEDED'),
        step('B', 'NOT_NEEDED'),
        step('C', 'NOT_NEEDED')
      ])
    )

    expect(row.text()).toContain('Flow finished, 3 steps not needed')
    expect(row.text()).not.toContain('Claimed, waiting for your review')
    expect(row.get('[data-testid="queue-steps-toggle"]').text()).toBe('1 of 1 step')
  })

  it('keeps the queue\'s own detail for a task that is not finished', () => {
    const row = mountRow(itemWith('RUNNING', SIX_STEPS, 'Waiting for you to answer a permission prompt.'))

    expect(row.text()).toContain('Waiting for you to answer a permission prompt.')
  })
})

describe('the status line with steps', () => {
  const now = Date.parse('2026-09-23T10:00:00Z')

  function lineOf(running: RunQueueItem): string {
    const queue = { ...EMPTY_RUN_QUEUE, state: 'RUNNING' as const, items: [running] }
    return mount(QueueStatusLine, { props: { queue, progress: runProgress(queue), now } }).get(
      '[data-testid="run-queue-status"]'
    ).text()
  }

  it('says which step is being worked', () => {
    expect(lineOf(itemWith('RUNNING', SIX_STEPS))).toContain('Running Report a, step 3 of 6: Three')
  })

  it('names only the task when it has no checklist', () => {
    expect(lineOf(itemWith('RUNNING', []))).toMatch(/^Running Report a(?!,)/)
  })
})
