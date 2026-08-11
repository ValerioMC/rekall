import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import AppButton from '@/components/ui/AppButton.vue'

describe('AppButton', () => {
  it('emits click when enabled', async () => {
    const wrapper = mount(AppButton, { slots: { default: 'Apply' } })

    await wrapper.trigger('click')

    expect(wrapper.emitted('click')).toHaveLength(1)
    expect(wrapper.text()).toBe('Apply')
  })

  it('is disabled while loading, so a slow action cannot be submitted twice', async () => {
    const wrapper = mount(AppButton, { props: { loading: true } })

    expect(wrapper.attributes('disabled')).toBeDefined()
    expect(wrapper.find('.animate-spin').exists()).toBe(true)
  })

  it('does not emit click when disabled', async () => {
    const wrapper = mount(AppButton, { props: { disabled: true } })

    await wrapper.trigger('click')

    expect(wrapper.emitted('click')).toBeUndefined()
  })
})
