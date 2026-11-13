import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'

const OPTIONS = [
  { value: 'comfortable', label: 'Comfortable' },
  { value: 'compact', label: 'Compact' }
] as const

describe('SegmentedControl', () => {
  it('marks the chosen option as pressed', () => {
    const wrapper = mount(SegmentedControl, { props: { options: OPTIONS, label: 'Density', modelValue: 'compact' } })
    expect(wrapper.get('[data-testid="segment-compact"]').attributes('aria-pressed')).toBe('true')
    expect(wrapper.get('[data-testid="segment-comfortable"]').attributes('aria-pressed')).toBe('false')
  })

  it('emits the option a press chooses', async () => {
    const wrapper = mount(SegmentedControl, { props: { options: OPTIONS, label: 'Density', modelValue: 'compact' } })
    await wrapper.get('[data-testid="segment-comfortable"]').trigger('click')
    expect(wrapper.emitted('update:modelValue')).toEqual([['comfortable']])
  })
})
