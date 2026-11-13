import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import { defineComponent } from 'vue'
import SubNav from '@/components/ui/SubNav.vue'

const ITEMS = [
  { id: 'needs', label: 'Needs me', count: 3, hot: true },
  { id: 'open', label: 'Open', count: 12 },
  { id: 'plain', label: 'Plain' }
] as const

describe('SubNav', () => {
  it('selects the active tab and shows a count only where an item has one', () => {
    const wrapper = mount(SubNav, { props: { items: ITEMS, label: 'Views', modelValue: 'open' } })
    expect(wrapper.get('[data-testid="sub-nav-open"]').attributes('aria-selected')).toBe('true')
    expect(wrapper.get('[data-testid="sub-nav-needs"]').attributes('aria-selected')).toBe('false')
    expect(wrapper.get('[data-testid="sub-nav-plain"]').find('[data-testid="sub-nav-count"]').exists()).toBe(false)
  })

  it('draws a count that waits on the reader as an amber pill, and a quiet one otherwise', () => {
    const wrapper = mount(SubNav, { props: { items: ITEMS, label: 'Views', modelValue: 'open' } })
    expect(wrapper.get('[data-testid="sub-nav-needs"] [data-testid="sub-nav-count"]').classes()).toContain('bg-accent')
    expect(wrapper.get('[data-testid="sub-nav-open"] [data-testid="sub-nav-count"]').classes()).not.toContain('bg-accent')
  })

  it('stays quiet when a hot count is zero', () => {
    const wrapper = mount(SubNav, { props: { items: [{ id: 'needs', label: 'Needs me', count: 0, hot: true }], label: 'Views', modelValue: 'needs' } })
    expect(wrapper.get('[data-testid="sub-nav-count"]').classes()).not.toContain('bg-accent')
  })

  it('emits the tab a press chooses', async () => {
    const wrapper = mount(SubNav, { props: { items: ITEMS, label: 'Views', modelValue: 'open' } })
    await wrapper.get('[data-testid="sub-nav-needs"]').trigger('click')
    expect(wrapper.emitted('update:modelValue')).toEqual([['needs']])
  })

  it('renders an item with a route as a link', async () => {
    const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/', component: defineComponent({ template: '<div />' }) }, { path: '/plan', component: defineComponent({ template: '<div />' }) }] })
    await router.push('/')
    const wrapper = mount(SubNav, {
      props: { items: [{ id: 'plan', label: 'Plan', to: '/plan' }], label: 'Task', modelValue: 'plan' },
      global: { plugins: [router] }
    })
    expect(wrapper.get('[data-testid="sub-nav-plan"]').element.tagName).toBe('A')
  })
})
