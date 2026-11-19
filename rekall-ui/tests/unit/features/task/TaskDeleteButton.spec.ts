import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import TaskDeleteButton from '@/features/task/components/TaskDeleteButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

const stub = { template: '<div />' }
let router: Router
let wrapper: VueWrapper
let deleteTask: ReturnType<typeof vi.fn>

function task(overrides: Partial<Task> = {}): Task {
  return { id: 't-1' as TaskId, code: 'RK-1', title: 'Report builder', documentCount: 2, trackerIssueKey: null, ...overrides } as Task
}

async function mountButton(subject: Task): Promise<void> {
  const pinia = createPinia()
  setActivePinia(pinia)
  deleteTask = vi.fn().mockResolvedValue(undefined)
  useConsoleStore().deleteTask = deleteTask as never
  router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/work', name: 'work', component: stub },
      { path: '/task/:id', name: 'task', component: stub }
    ]
  })
  await router.push('/task/t-1')
  await router.isReady()
  wrapper = mount(TaskDeleteButton, { attachTo: document.body, props: { task: subject }, global: { plugins: [pinia, router] } })
}

function confirmButton(): HTMLButtonElement {
  return [...document.body.querySelectorAll('button')].find((button) => button.textContent?.trim() === 'Delete task')!
}

describe('TaskDeleteButton', () => {
  beforeEach(() => document.body.replaceChildren())
  afterEach(() => wrapper?.unmount())

  it('deletes nothing until the confirm is accepted, then returns to Work', async () => {
    await mountButton(task())

    await wrapper.get('[data-testid="task-delete"]').trigger('click')
    expect(document.body.textContent).toContain('2 notes deleted')
    expect(deleteTask).not.toHaveBeenCalled()

    confirmButton().click()
    await flushPromises()

    expect(deleteTask).toHaveBeenCalledWith('t-1')
    expect(router.currentRoute.value.name).toBe('work')
  })

  it('says the task manager issue is deleted too', async () => {
    await mountButton(task({ trackerIssueKey: 'RK-77' }))

    await wrapper.get('[data-testid="task-delete"]').trigger('click')

    expect(document.body.textContent).toContain('Issue RK-77 is deleted from the project')
  })

  it('stays on the task when the delete fails', async () => {
    await mountButton(task())
    deleteTask.mockRejectedValue(new Error('boom'))

    await wrapper.get('[data-testid="task-delete"]').trigger('click')
    confirmButton().click()
    await flushPromises()

    expect(router.currentRoute.value.name).toBe('task')
  })
})
