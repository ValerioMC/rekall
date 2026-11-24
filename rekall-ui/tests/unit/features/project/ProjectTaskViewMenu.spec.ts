import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount, type VueWrapper } from '@vue/test-utils'
import ProjectTaskViewMenu from '@/features/project/components/ProjectTaskViewMenu.vue'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import { installStorage } from '../../support/memoryStorage'

let wrapper: VueWrapper | null = null

function render(showsQuickTasks: boolean): VueWrapper {
  wrapper = mount(ProjectTaskViewMenu, { props: { showsQuickTasks }, global: { plugins: [createPinia()] }, attachTo: document.body })
  return wrapper
}

beforeEach(() => {
  installStorage()
  setActivePinia(createPinia())
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  delete (window as { localStorage?: unknown }).localStorage
})

describe('the project tasks view menu', () => {
  it('stays closed until its key is pressed and closes on Escape', async () => {
    const menu = render(false)
    expect(menu.find('[data-testid="task-view-panel"]').exists()).toBe(false)

    await menu.get('[data-testid="task-view-trigger"]').trigger('click')
    expect(menu.find('[data-testid="task-view-panel"]').exists()).toBe(true)

    await menu.get('[data-testid="task-view-menu"]').trigger('keydown', { key: 'Escape' })
    expect(menu.find('[data-testid="task-view-panel"]').exists()).toBe(false)
  })

  it('sets the row density in the project tasks store', async () => {
    const menu = render(false)
    await menu.get('[data-testid="task-view-trigger"]').trigger('click')

    await menu.get('[data-testid="segment-compact"]').trigger('click')

    expect(useProjectTasksStore().density).toBe('compact')
  })

  it('offers the quick-task filter only when there are quick tasks', async () => {
    const withQuickTasks = render(true)
    await withQuickTasks.get('[data-testid="task-view-trigger"]').trigger('click')
    expect(withQuickTasks.find('[data-testid="hide-quick-tasks"]').exists()).toBe(true)
    withQuickTasks.unmount()

    const withoutQuickTasks = render(false)
    await withoutQuickTasks.get('[data-testid="task-view-trigger"]').trigger('click')
    expect(withoutQuickTasks.find('[data-testid="hide-quick-tasks"]').exists()).toBe(false)
  })
})
