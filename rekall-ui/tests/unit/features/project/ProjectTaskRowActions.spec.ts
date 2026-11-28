import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import ProjectTasksSection from '@/features/project/components/ProjectTasksSection.vue'
import { router } from '@/router'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import { project, task } from './project-task-fixtures'

function seed() {
  setActivePinia(createPinia())
  const store = useConsoleStore()
  store.isLoading = false
  store.projects = [project('p1', 'Vega')]
  store.tasks = [
    task('PR-1', { title: 'Write the docs', status: 'TODO' }),
    task('PR-2', { title: 'Fix the login', quick: true })
  ]
  const setTaskStatus = vi.spyOn(store, 'setTaskStatus').mockResolvedValue()
  useProjectTasksStore().view = 'open'
  return { setTaskStatus }
}

async function mountTasks() {
  const wrapper = mount(ProjectTasksSection, { attachTo: document.body, global: { plugins: [router] } })
  await flushPromises()
  return wrapper
}

const rowOf = (wrapper: Awaited<ReturnType<typeof mountTasks>>, title: string) =>
  wrapper.findAll('[data-testid="task-row"]').find((candidate) => candidate.text().includes(title))!

describe('the actions on a ledger row', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
    document.body.innerHTML = ''
  })

  it('changes the status from the status column without opening the row', async () => {
    const { setTaskStatus } = seed()
    const wrapper = await mountTasks()
    const row = rowOf(wrapper, 'Write the docs')

    await row.get('[data-testid="task-status"]').trigger('click')
    const blocked = row.findAll('[data-testid="popover-menu-item"]').find((item) => item.text() === 'Blocked')!
    await blocked.trigger('click')
    await flushPromises()

    expect(setTaskStatus).toHaveBeenCalledWith('PR-1', 'BLOCKED')
    expect(useToastStore().toasts.some((toast) => toast.message === 'Status set to blocked')).toBe(true)
    expect(wrapper.find('[data-testid="task-peek"]').exists()).toBe(false)
  })

  it('marks the stored status as current in the menu', async () => {
    seed()
    const wrapper = await mountTasks()
    const row = rowOf(wrapper, 'Write the docs')

    await row.get('[data-testid="task-status"]').trigger('click')

    const current = row.findAll('[data-testid="popover-menu-item"]').filter((item) => item.attributes('aria-current') === 'true')
    expect(current.map((item) => item.text())).toEqual(['To do'])
  })

  it('opens the task from its title in one press, without opening the row', async () => {
    seed()
    const push = vi.spyOn(router, 'push').mockResolvedValue(undefined)
    const wrapper = await mountTasks()

    await rowOf(wrapper, 'Write the docs').get('[data-testid="task-row-title"]').trigger('click', { button: 0 })

    expect(push).toHaveBeenCalledWith('/task/PR-1/description')
    expect(wrapper.find('[data-testid="task-peek"]').exists()).toBe(false)
  })

  it('opens the preview from the arrow at the start of the row', async () => {
    seed()
    const wrapper = await mountTasks()
    const row = rowOf(wrapper, 'Write the docs')
    expect(row.element.firstElementChild).toBe(row.get('[data-testid="task-row-preview"]').element)

    await row.get('[data-testid="task-row-preview"]').trigger('click')

    expect(wrapper.find('[data-testid="task-peek"]').exists()).toBe(true)
  })

  it('badges a quick task and no other', async () => {
    seed()
    const wrapper = await mountTasks()

    expect(rowOf(wrapper, 'Fix the login').find('[data-testid="task-row-quick"]').text()).toBe('Quick')
    expect(rowOf(wrapper, 'Write the docs').find('[data-testid="task-row-quick"]').exists()).toBe(false)
  })

  it('deletes a task from its row after the confirm, without opening the row', async () => {
    seed()
    const store = useConsoleStore()
    const deleteTask = vi.spyOn(store, 'deleteTask').mockResolvedValue()
    const wrapper = await mountTasks()
    const row = rowOf(wrapper, 'Fix the login')

    await row.get('[data-testid="task-delete"]').trigger('click')
    expect(deleteTask).not.toHaveBeenCalled()
    expect(useProjectTasksStore().peekedTaskId).toBeNull()
    const confirm = Array.from(document.body.querySelectorAll('button')).find((key) => key.textContent?.trim() === 'Delete task')!
    confirm.click()
    await flushPromises()

    expect(deleteTask).toHaveBeenCalledWith('PR-2')
  })
})
