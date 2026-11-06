import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import FeedbackComposer from '@/features/feedback/components/FeedbackComposer.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import { useConsoleStore } from '@/stores/console.store'
import { TASK } from './fixtures'

function composer() {
  const store = useConsoleStore()
  store.selectedTaskId = TASK
  const add = vi.spyOn(store, 'addFeedback').mockResolvedValue()
  const wrapper = mount(FeedbackComposer, { global: { stubs: { AppMarkdownEditor: true } } })
  return { wrapper, add }
}

async function write(wrapper: ReturnType<typeof composer>['wrapper'], text: string): Promise<void> {
  wrapper.getComponent(AppMarkdownEditor).vm.$emit('update:modelValue', text)
  await wrapper.vm.$nextTick()
}

describe('the feedback composer', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('cannot add while nothing is written', () => {
    const { wrapper } = composer()

    expect(wrapper.get('[data-testid="feedback-add"]').attributes('disabled')).toBeDefined()
  })

  it('adds the text as feedback on the work and leaves the editor empty', async () => {
    const { wrapper, add } = composer()
    await write(wrapper, 'The totals are wrong')

    await wrapper.get('form').trigger('submit')
    await vi.waitFor(() => expect(add).toHaveBeenCalledWith(TASK, 'IMPLEMENTATION', 'The totals are wrong', null))

    await vi.waitFor(() => expect(wrapper.getComponent(AppMarkdownEditor).props('modelValue')).toBe(''))
  })

  it('adds with Cmd+Enter and with Ctrl+Enter', async () => {
    const { wrapper, add } = composer()

    await write(wrapper, 'one')
    await wrapper.get('form').trigger('keydown', { key: 'Enter', metaKey: true })
    await vi.waitFor(() => expect(add).toHaveBeenCalledTimes(1))

    await write(wrapper, 'two')
    await wrapper.get('form').trigger('keydown', { key: 'Enter', ctrlKey: true })
    await vi.waitFor(() => expect(add).toHaveBeenCalledTimes(2))
  })

  it('switches what the feedback is about', async () => {
    const { wrapper, add } = composer()
    await wrapper.get('[data-testid="feedback-kind-PLAN"]').trigger('click')
    await write(wrapper, 'Split step 2')

    await wrapper.get('form').trigger('submit')

    await vi.waitFor(() => expect(add).toHaveBeenCalledWith(TASK, 'PLAN', 'Split step 2', null))
  })

  it('says which applied entry it adds detail to, sends it along, and forgets it after', async () => {
    const { wrapper, add } = composer()
    ;(wrapper.vm as unknown as { followUp: (seq: number) => void }).followUp(3)
    await wrapper.vm.$nextTick()
    expect(wrapper.get('[data-testid="feedback-following"]').text()).toContain('Adding detail to #3')
    await write(wrapper, 'Still wrong for refunds')

    await wrapper.get('form').trigger('submit')

    await vi.waitFor(() => expect(add).toHaveBeenCalledWith(TASK, 'IMPLEMENTATION', 'Still wrong for refunds', 3))
    await vi.waitFor(() => expect(wrapper.find('[data-testid="feedback-following"]').exists()).toBe(false))
  })

  it('lets the person drop the entry it was aimed at', async () => {
    const { wrapper } = composer()
    ;(wrapper.vm as unknown as { followUp: (seq: number) => void }).followUp(3)
    await wrapper.vm.$nextTick()

    await wrapper.get('[data-testid="feedback-following-drop"]').trigger('click')

    expect(wrapper.find('[data-testid="feedback-following"]').exists()).toBe(false)
  })

  it('gives the editor room for several lines', () => {
    const { wrapper } = composer()

    expect(wrapper.getComponent(AppMarkdownEditor).props('height')).toBe('176px')
  })
})
