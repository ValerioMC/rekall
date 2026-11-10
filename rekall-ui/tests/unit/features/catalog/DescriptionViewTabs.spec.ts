import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import DescriptionViewTabs from '@/features/catalog/components/DescriptionViewTabs.vue'

function tabs(count: number, attention: boolean, modelValue: 'description' | 'revisions' = 'description', running = false) {
  return mount(DescriptionViewTabs, { props: { modelValue, count, attention, running } })
}

describe('the tabs of the description window', () => {
  it('marks the open one and switches to the other', async () => {
    const wrapper = tabs(0, false)

    expect(wrapper.get('[data-testid="view-description"]').attributes('aria-pressed')).toBe('true')
    await wrapper.get('[data-testid="view-revisions"]').trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([['revisions']])
  })

  it('carries how many revisions there are, and nothing when there are none', () => {
    expect(tabs(3, false).get('[data-testid="view-revisions-count"]').text()).toBe('3')
    expect(tabs(0, false).find('[data-testid="view-revisions-count"]').exists()).toBe(false)
  })

  it('lights up while a revision needs a run', () => {
    expect(tabs(1, true).find('[data-testid="view-revisions-attention"]').exists()).toBe(true)
    expect(tabs(1, false).find('[data-testid="view-revisions-attention"]').exists()).toBe(false)
  })

  it('pings the light only while a session is on a revision', () => {
    expect(tabs(1, true, 'description', true).find('[data-testid="view-revisions-running"]').exists()).toBe(true)
    expect(tabs(1, true).find('[data-testid="view-revisions-running"]').exists()).toBe(false)
  })
})
