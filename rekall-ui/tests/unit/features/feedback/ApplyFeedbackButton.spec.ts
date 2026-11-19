import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ApplyFeedbackButton from '@/features/feedback/components/ApplyFeedbackButton.vue'
import type { RevisionRunPhase } from '@/features/feedback/composables/useRevisionRun'

function key(phase: RevisionRunPhase, count = 2) {
  return mount(ApplyFeedbackButton, { props: { phase, count, hint: 'a hint' } })
}

describe('the Run revision key', () => {
  it('offers the run with the count of waiting revisions, and takes a press', async () => {
    const wrapper = key('idle', 3)

    expect(wrapper.text()).toContain('Run revision')
    expect(wrapper.get('[data-testid="apply-feedback-count"]').text()).toBe('3')
    await wrapper.trigger('click')
    expect(wrapper.emitted('press')).toHaveLength(1)
  })

  it.each(['launching', 'waiting', 'working', 'done'] as const)('takes no press while %s', async (phase) => {
    const wrapper = key(phase)

    await wrapper.trigger('click')

    expect(wrapper.emitted('press')).toBeUndefined()
    expect(wrapper.attributes('aria-disabled')).toBe('true')
  })

  it('turns rings while the server answers, shows Working during a pass and stamps a seal when it is finished', () => {
    expect(key('waiting').find('.run-ring').exists()).toBe(true)
    expect(key('working').text()).toContain('Working')
    expect(key('working').attributes('aria-label')).toBe('Working')
    expect(key('done').find('.run-seal').exists()).toBe(true)
    expect(key('idle').find('.run-seal').exists()).toBe(false)
  })
})
