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

  it('draws every solid variant as a control-cornered key, with no outline, and a ghost flat at rest', () => {
    const primary = mount(AppButton, { props: { variant: 'primary' } })
    const secondary = mount(AppButton)
    const danger = mount(AppButton, { props: { variant: 'danger' } })
    const ghost = mount(AppButton, { props: { variant: 'ghost' } })

    expect(primary.classes()).toContain('key-gold')
    expect(secondary.classes()).toContain('key-slate')
    expect(danger.classes()).toContain('key-danger')
    expect(ghost.classes()).not.toContain('key-slate')
    for (const button of [primary, secondary, danger, ghost]) {
      expect(button.classes()).toContain('rounded-control')
      expect(button.classes()).toContain('border-0')
    }
  })

  it('announces a slow action as busy while it spins', () => {
    const idle = mount(AppButton)
    const busy = mount(AppButton, { props: { loading: true } })

    expect(idle.attributes('aria-busy')).toBeUndefined()
    expect(busy.attributes('aria-busy')).toBe('true')
  })

  it('lights a toggle that is on and says so, and leaves a plain button without a pressed state', () => {
    const plain = mount(AppButton, { props: { variant: 'quiet' } })
    const off = mount(AppButton, { props: { variant: 'quiet', pressed: false } })
    const on = mount(AppButton, { props: { variant: 'quiet', pressed: true } })

    expect(plain.attributes('aria-pressed')).toBeUndefined()
    expect(plain.classes()).toContain('key-quiet')
    expect(off.attributes('aria-pressed')).toBe('false')
    expect(off.classes()).toContain('key-quiet')
    expect(on.attributes('aria-pressed')).toBe('true')
    expect(on.classes()).toContain('key-lit')
    expect(on.classes()).not.toContain('key-quiet')
  })

  it('draws an icon key square, so its glyph sits in a 32px target', () => {
    const wrapper = mount(AppButton, { props: { variant: 'quiet', size: 'icon' }, attrs: { 'aria-label': 'Settings' } })

    expect(wrapper.classes()).toContain('size-8')
    expect(wrapper.attributes('aria-label')).toBe('Settings')
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
