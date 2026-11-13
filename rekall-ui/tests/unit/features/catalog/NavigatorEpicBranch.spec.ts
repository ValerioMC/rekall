import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import NavigatorEpicBranch from '@/features/catalog/components/epic/NavigatorEpicBranch.vue'
import { taskFixture } from '../../support/taskFixture'
import type { EpicBranch } from '@/features/catalog/lib/partition-tasks'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

const epic: Task = { ...taskFixture('e' as TaskId), kind: 'EPIC', code: 'RK-1', title: 'Checkout' }
const pay: Task = { ...taskFixture('a' as TaskId), epicId: epic.id, code: 'RK-3', title: 'Pay' }
const refund: Task = { ...taskFixture('b' as TaskId), epicId: epic.id, code: 'RK-2', title: 'Refund', status: 'DONE' }
const branch: EpicBranch = { epic, children: [pay, refund] }

function mountBranch(expanded: boolean, selectedTaskId: TaskId | null = null) {
  return mount(NavigatorEpicBranch, {
    props: { branch, expanded, selectedTaskId, runningTaskIds: new Set<TaskId>() }
  })
}

beforeEach(() => setActivePinia(createPinia()))

describe('NavigatorEpicBranch', () => {
  it('shows the epic with its code and how many of its tasks are done', () => {
    const wrapper = mountBranch(false)

    expect(wrapper.get('[data-testid="epic-row"]').text()).toContain('Checkout')
    expect(wrapper.get('[data-testid="epic-code"]').text()).toBe('RK-1')
    expect(wrapper.get('[data-testid="epic-progress"]').text()).toBe('1/2')
  })

  it('keeps its tasks out of reach while it is closed, and shows them once open', () => {
    const closed = mountBranch(false)
    const open = mountBranch(true)

    expect(closed.get('.app-collapse').attributes('inert')).toBeDefined()
    expect(closed.get('[data-testid="epic-toggle"]').attributes('aria-expanded')).toBe('false')
    expect(open.get('.app-collapse').attributes('inert')).toBeUndefined()
    expect(open.get('[data-testid="epic-toggle"]').attributes('aria-expanded')).toBe('true')
    expect(open.findAll('[data-testid="task-row"]').map((row) => row.text())).toEqual([
      expect.stringContaining('Pay'),
      expect.stringContaining('Refund')
    ])
  })

  it('opens and closes only from the chevron; the title selects the epic', async () => {
    const wrapper = mountBranch(false)

    await wrapper.get('[data-testid="epic-toggle"]').trigger('click')
    await wrapper.get('.epic-title').trigger('click')

    expect(wrapper.emitted('toggle')).toHaveLength(1)
    expect(wrapper.emitted('select')).toEqual([[epic.id]])
  })

  it('selects a task of the epic by its own id', async () => {
    const wrapper = mountBranch(true)

    await wrapper.findAll('[data-testid="task-row"]')[0]!.trigger('click')

    expect(wrapper.emitted('select')).toEqual([[pay.id]])
  })

  it('has no chevron when no task belongs to it', () => {
    const wrapper = mount(NavigatorEpicBranch, {
      props: { branch: { epic, children: [] }, expanded: false, selectedTaskId: null, runningTaskIds: new Set<TaskId>() }
    })

    expect(wrapper.find('[data-testid="epic-toggle"]').exists()).toBe(false)
    expect(wrapper.text()).toContain('No task belongs to it yet')
  })
})
