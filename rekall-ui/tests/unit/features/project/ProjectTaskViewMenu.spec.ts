import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount, type VueWrapper } from '@vue/test-utils'
import ProjectTaskViewMenu from '@/features/project/components/ProjectTaskViewMenu.vue'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import { installStorage } from '../../support/memoryStorage'

let wrapper: VueWrapper | null = null

async function renderOpen(): Promise<VueWrapper> {
  wrapper = mount(ProjectTaskViewMenu, { attachTo: document.body })
  await wrapper.get('[data-testid="task-view-trigger"]').trigger('click')
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
    wrapper = mount(ProjectTaskViewMenu, { attachTo: document.body })
    expect(wrapper.find('[data-testid="task-view-panel"]').exists()).toBe(false)

    await wrapper.get('[data-testid="task-view-trigger"]').trigger('click')
    expect(wrapper.find('[data-testid="task-view-panel"]').exists()).toBe(true)

    await wrapper.get('[data-testid="task-view-menu"]').trigger('keydown', { key: 'Escape' })
    expect(wrapper.find('[data-testid="task-view-panel"]').exists()).toBe(false)
  })

  it('sets the row density in the project tasks store', async () => {
    const menu = await renderOpen()

    await menu.get('[data-testid="segment-compact"]').trigger('click')

    expect(useProjectTasksStore().density).toBe('compact')
  })

  it('turns the side preview off and says it needs more width', async () => {
    const menu = await renderOpen()

    await menu.get('[data-testid="task-side-preview"]').setValue(false)

    expect(useProjectTasksStore().sidePreview).toBe(false)
    expect(menu.text()).toContain('needs more width')
  })

  it('takes a column away and gives it back', async () => {
    const menu = await renderOpen()

    await menu.get('[data-testid="task-column-tokens"]').trigger('change')
    expect(useProjectTasksStore().hiddenColumns.has('tokens')).toBe(true)

    await menu.get('[data-testid="task-column-tokens"]').trigger('change')
    expect(useProjectTasksStore().hiddenColumns.has('tokens')).toBe(false)
  })
})
