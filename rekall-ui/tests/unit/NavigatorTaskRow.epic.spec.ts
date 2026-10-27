import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import NavigatorTaskRow from '@/components/console/NavigatorTaskRow.vue'
import { useConsoleStore } from '@/stores/console.store'
import { taskFixture } from './support/taskFixture'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/** The mark on a task's row that says which epic it belongs to. */
const epic: Task = { ...taskFixture('checkout' as TaskId), kind: 'EPIC', title: 'Checkout' }
const member: Task = { ...taskFixture('pay' as TaskId), epicId: epic.id }

function mountRow(task: Task) {
  return mount(NavigatorTaskRow, {
    props: { task, selected: false, running: false, showContext: false, showCompany: false }
  })
}

beforeEach(() => setActivePinia(createPinia()))

describe('NavigatorTaskRow epic', () => {
  it('names the epic a task belongs to', () => {
    useConsoleStore().tasks = [epic, member]

    expect(mountRow(member).get('[data-testid="task-epic"]').text()).toBe('Checkout')
  })

  it('shows no epic on a task that belongs to none', () => {
    useConsoleStore().tasks = [epic, member]

    expect(mountRow(taskFixture('login' as TaskId)).find('[data-testid="task-epic"]').exists()).toBe(false)
  })
})
