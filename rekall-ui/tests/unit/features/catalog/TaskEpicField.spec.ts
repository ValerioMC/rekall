import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import TaskEpicField from '@/features/catalog/components/record/TaskEpicField.vue'
import { useConsoleStore } from '@/stores/console.store'
import { taskFixture } from '../../support/taskFixture'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

function epic(id: string, title: string): Task {
  return { ...taskFixture(id as TaskId), kind: 'EPIC', title, label: id, anchor: `task:${id}` }
}

const epics = [epic('billing', 'Billing'), epic('exports', 'Exports'), epic('search', 'Search')]

function render(modelValue: TaskId | null = null) {
  return mount(TaskEpicField, { props: { modelValue, epics } })
}

async function opened(modelValue: TaskId | null = null) {
  const wrapper = render(modelValue)
  await wrapper.find('[data-testid="epic-field-trigger"]').trigger('click')
  return wrapper
}

function optionTexts(wrapper: ReturnType<typeof render>): string[] {
  return wrapper.findAll('[data-testid="epic-field-option"]').map((option) => option.text())
}

beforeEach(() => {
  setActivePinia(createPinia())
  useConsoleStore().tasks = [...epics, { ...taskFixture('t1' as TaskId), epicId: 'exports' as TaskId }]
})

describe('TaskEpicField', () => {
  it('names the current epic while closed, with no list showing', () => {
    const wrapper = render('exports' as TaskId)

    expect(wrapper.find('[data-testid="epic-field-title"]').text()).toBe('Exports')
    expect(wrapper.text()).toContain('task:exports')
    expect(wrapper.text()).toContain('1 tasks')
    expect(wrapper.find('[data-testid="epic-field-search"]').exists()).toBe(false)
  })

  it('opens on every epic of the project, after No epic', async () => {
    const wrapper = await opened()

    expect(optionTexts(wrapper)).toEqual(['No epic', expect.stringContaining('Billing'), expect.stringContaining('Exports'), expect.stringContaining('Search')])
  })

  it('narrows the list as the person types', async () => {
    const wrapper = await opened()

    await wrapper.find('[data-testid="epic-field-search"]').setValue('exp')

    expect(optionTexts(wrapper)).toEqual([expect.stringContaining('Exports')])
  })

  it('says so when nothing matches', async () => {
    const wrapper = await opened()

    await wrapper.find('[data-testid="epic-field-search"]').setValue('zzz')

    expect(wrapper.find('[data-testid="epic-field-empty"]').text()).toContain('zzz')
  })

  it('picks the highlighted epic with Enter, then folds away', async () => {
    const wrapper = await opened()
    const search = wrapper.find('[data-testid="epic-field-search"]')

    await search.setValue('search')
    await search.trigger('keydown', { key: 'Enter' })

    expect(wrapper.emitted('update:modelValue')).toEqual([['search']])
    expect(wrapper.find('[data-testid="epic-field-search"]').exists()).toBe(false)
  })

  it('takes the task out of its epic when No epic is clicked, and reports open and close', async () => {
    const wrapper = await opened('billing' as TaskId)

    await wrapper.findAll('[data-testid="epic-field-option"]')[0]!.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([[null]])
    expect(wrapper.emitted('open-change')).toEqual([[true], [false]])
  })
})
