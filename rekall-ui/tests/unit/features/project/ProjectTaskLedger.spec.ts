import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import ProjectTasksSection from '@/features/project/components/ProjectTasksSection.vue'
import { router } from '@/router'
import { useConsoleStore } from '@/stores/console.store'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import type { Task } from '@/features/catalog/model/catalog'
import { project, task } from './project-task-fixtures'

async function mountList(tasks: Task[]) {
  setActivePinia(createPinia())
  const store = useConsoleStore()
  store.isLoading = false
  store.projects = [project('p1', 'Vega')]
  store.tasks = tasks
  const projectTasks = useProjectTasksStore()
  projectTasks.view = 'all'
  const wrapper = mount(ProjectTasksSection, { attachTo: document.body, global: { plugins: [router] } })
  await flushPromises()
  return { wrapper, projectTasks }
}

const many = (count: number) => Array.from({ length: count }, (_, index) => task(`PR-${index + 1}`))

describe('the task list of a project', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, value: undefined })
    document.body.innerHTML = ''
  })

  it('lists a page at a time and walks the pages', async () => {
    const { wrapper } = await mountList(many(30))
    expect(wrapper.findAll('[data-testid="task-row"]')).toHaveLength(25)
    expect(wrapper.get('[data-testid="task-pager-range"]').text()).toBe('1–25 of 30 tasks')

    await wrapper.get('[data-testid="task-pager-next"]').trigger('click')

    expect(wrapper.findAll('[data-testid="task-row"]')).toHaveLength(5)
    expect(wrapper.get('[data-testid="task-pager-range"]').text()).toBe('26–30 of 30 tasks')
  })

  it('holds as many rows as the reader asks for, with no page numbers for one page', async () => {
    const { wrapper } = await mountList(many(30))

    await wrapper.get('[data-testid="segment-50"]').trigger('click')

    expect(wrapper.findAll('[data-testid="task-row"]')).toHaveLength(30)
    expect(wrapper.find('[data-testid="task-pager-next"]').exists()).toBe(false)
  })

  it('narrows the list from a filter and clears it again', async () => {
    const { wrapper } = await mountList([task('PR-1', { quick: true }), task('PR-2')])

    await wrapper.get('[data-testid="task-filter-type"] [data-testid="task-filter-trigger"]').trigger('click')
    await wrapper.get('[data-testid="task-filter-value-quick"]').trigger('change')

    expect(wrapper.findAll('[data-testid="task-row"]')).toHaveLength(1)
    expect(wrapper.get('[data-testid="task-filter-type"] [data-testid="task-filter-summary"]').text()).toBe('Quick')

    await wrapper.get('[data-testid="task-clear"]').trigger('click')

    expect(wrapper.findAll('[data-testid="task-row"]')).toHaveLength(2)
  })

  it('offers to clear the search when nothing matches it', async () => {
    const { wrapper } = await mountList(many(2))

    await wrapper.get('[data-testid="task-filter"]').setValue('zzz')
    expect(wrapper.find('[data-testid="task-pager"]').exists()).toBe(false)

    await wrapper.get('[data-testid="task-no-match-clear"]').trigger('click')

    expect(wrapper.findAll('[data-testid="task-row"]')).toHaveLength(2)
  })

  it('orders by a column head, one way and then the other', async () => {
    const { wrapper } = await mountList([task('PR-1', { title: 'Zeta' }), task('PR-2', { title: 'Alpha' })])
    const titles = () => wrapper.findAll('[data-testid="task-row-title"]').map((title) => title.text())
    const head = () => wrapper.get('[data-testid="task-head-title"] button')

    await head().trigger('click')
    expect(titles()).toEqual(['Alpha', 'Zeta'])

    await head().trigger('click')
    expect(titles()).toEqual(['Zeta', 'Alpha'])
    expect(wrapper.get('[data-testid="task-head-title"]').attributes('aria-sort')).toBe('descending')
  })

  it('gains columns and a preview beside the list when it is wide', async () => {
    const { wrapper, projectTasks } = await mountList([task('PR-1', { description: 'Do it', documentCount: 2 })])
    expect(wrapper.find('[data-testid="task-row-step"]').exists()).toBe(false)

    projectTasks.measure(2400)
    await nextTick()

    expect(wrapper.get('[data-testid="task-row-step"]').text()).toBe('Do it')
    expect(wrapper.get('[data-testid="task-row-notes"]').text()).toBe('2')
    expect(wrapper.get('[data-testid="task-preview-title"]').text()).toBe('Task PR-1')
  })

  it('previews the row pressed beside the list instead of opening it under the row', async () => {
    const { wrapper, projectTasks } = await mountList(many(2))
    projectTasks.measure(2400)
    await nextTick()

    await wrapper.findAll('[data-testid="task-row"]')[1]!.trigger('click')

    expect(wrapper.get('[data-testid="task-preview-title"]').text()).toBe('Task PR-1')
    expect(wrapper.findAll('[data-testid="task-peek"]')).toHaveLength(1)
  })

  it('counts a claim in the status pill', async () => {
    const { wrapper } = await mountList([task('PR-1', { stepCount: 2 })])
    useConsoleStore().steps = [
      { id: 's1', taskId: 'PR-1', title: 'One', state: 'CLAIMED', position: 1, visits: [], links: [], passes: [], updatedAt: '2026-09-20T09:00:00Z' } as never
    ]
    await nextTick()

    expect(wrapper.get('[data-testid="task-status"]').text()).toBe('1 to review')
  })
})
