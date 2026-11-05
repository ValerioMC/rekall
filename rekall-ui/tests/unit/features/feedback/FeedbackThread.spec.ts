import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import FeedbackThread from '@/features/feedback/components/FeedbackThread.vue'
import { applied, entry } from './fixtures'

const stubs = { AppMarkdownEditor: true }

describe('the thread under the description', () => {
  it('invites the first message when there is none', () => {
    const wrapper = mount(FeedbackThread, { props: { entries: [] }, global: { stubs } })

    expect(wrapper.find('[data-testid="feedback-empty"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="feedback-thread"]').exists()).toBe(false)
  })

  it('lists the messages in the order given, oldest first, and passes their requests on', async () => {
    const wrapper = mount(FeedbackThread, { props: { entries: [applied(1, 'Done.'), entry(2)] }, global: { stubs } })

    const seqs = wrapper.findAll('[data-testid="feedback-entry"]').map((item) => item.attributes('data-state'))
    expect(seqs).toEqual(['APPLIED', 'PENDING'])

    await wrapper.get('[data-testid="feedback-delete"]').trigger('click')

    expect(wrapper.emitted('remove')).toEqual([[entry(2)]])
  })
})
