import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import AnchorBar from '@/components/console/AnchorBar.vue'
import { useConsoleStore } from '@/stores/console.store'
import { mountWithHeaderChrome } from './support/mountWithHeaderChrome'
import type { Task } from '@/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

const task = {
  id: 't1' as TaskId,
  title: 'Report builder',
  projectId: 'p1' as ProjectId,
  anchor: 'project:vega task:report-builder'
} as Task

function render() {
  return mountWithHeaderChrome(AnchorBar, {
    global: { stubs: { HeaderStatus: true } }
  })
}

describe('the anchor bar, at rest', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('shows the selected task as its anchors, the last one lit', () => {
    const store = useConsoleStore()
    store.tasks = [task]
    store.selectedTaskId = task.id

    const wrapper = render()
    const chips = wrapper.findAll('[data-testid="anchor-location"] .anchor-chip')

    expect(chips.map((chip) => chip.text())).toEqual(['project:vega', 'task:report-builder'])
    expect(chips[1]!.classes()).toContain('text-anchor-strong')
    expect(wrapper.get('[data-testid="anchor-input"]').attributes('placeholder')).toBe('')
    wrapper.unmount()
  })

  it('turns back into a search when focused', async () => {
    const store = useConsoleStore()
    store.tasks = [task]
    store.selectedTaskId = task.id

    const wrapper = render()
    await wrapper.get('[data-testid="anchor-input"]').trigger('focus')

    expect(wrapper.find('[data-testid="anchor-location"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('shows the search hint when nothing is selected or scoped', () => {
    const wrapper = render()

    expect(wrapper.find('[data-testid="anchor-location"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="anchor-input"]').attributes('placeholder')).toBe('project:vega task:setup')
    wrapper.unmount()
  })
})
