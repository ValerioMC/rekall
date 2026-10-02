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

  it('draws every solid variant as a square keycap, with no outline, and a ghost flat at rest', () => {
    const primary = mount(AppButton, { props: { variant: 'primary' } })
    const secondary = mount(AppButton)
    const danger = mount(AppButton, { props: { variant: 'danger' } })
    const ghost = mount(AppButton, { props: { variant: 'ghost' } })

    expect(primary.classes()).toContain('key-gold')
    expect(secondary.classes()).toContain('key-slate')
    expect(danger.classes()).toContain('key-danger')
    expect(ghost.classes()).not.toContain('key-slate')
    for (const button of [primary, secondary, danger, ghost]) {
      expect(button.classes()).toContain('rounded-[3px]')
      expect(button.classes()).toContain('border-0')
    }
  })

  it('does not emit click when disabled', async () => {
    const wrapper = mount(AppButton, { props: { disabled: true } })

    await wrapper.trigger('click')

    expect(wrapper.emitted('click')).toBeUndefined()
  })

  it('keeps a repeated delete grey until it is under the pointer', () => {
    const wrapper = mount(AppButton, { props: { variant: 'danger-quiet' }, slots: { default: 'Delete' } })

    expect(wrapper.classes()).toContain('text-text-muted')
    expect(wrapper.classes()).toContain('hover:text-danger')
    expect(wrapper.classes()).not.toContain('text-danger')
  })
})
