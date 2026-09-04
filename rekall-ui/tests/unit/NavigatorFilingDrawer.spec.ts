import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import NavigatorFilingDrawer from '@/components/console/NavigatorFilingDrawer.vue'

describe('NavigatorFilingDrawer', () => {
  it('shows the count and keeps the body out of the DOM while closed', () => {
    const wrapper = mount(NavigatorFilingDrawer, {
      props: { count: 6, open: false },
      slots: { default: '<span class="row">row</span>' }
    })

    expect(wrapper.get('[data-testid="filing-drawer-toggle"]').text()).toContain('6 filed')
    expect(wrapper.get('[data-testid="filing-drawer-toggle"]').attributes('aria-expanded')).toBe(
      'false'
    )
    expect(wrapper.find('.row').exists()).toBe(false)
  })

  it('renders the slot and marks itself expanded while open', () => {
    const wrapper = mount(NavigatorFilingDrawer, {
      props: { count: 2, open: true },
      slots: { default: '<span class="row">row</span>' }
    })

    expect(wrapper.get('[data-testid="filing-drawer-toggle"]').attributes('aria-expanded')).toBe(
      'true'
    )
    expect(wrapper.find('.row').exists()).toBe(true)
  })

  it('emits toggle when the disclosure row is clicked', async () => {
    const wrapper = mount(NavigatorFilingDrawer, { props: { count: 1, open: false } })

    await wrapper.get('[data-testid="filing-drawer-toggle"]').trigger('click')

    expect(wrapper.emitted('toggle')).toHaveLength(1)
  })
})
