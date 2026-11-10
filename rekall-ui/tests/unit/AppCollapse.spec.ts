import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import AppCollapse from '@/components/ui/AppCollapse.vue'

describe('AppCollapse', () => {
  it('keeps its content reachable while open', () => {
    const wrapper = mount(AppCollapse, { props: { open: true }, slots: { default: '<button>Task</button>' } })

    expect(wrapper.attributes('data-open')).toBe('true')
    expect(wrapper.attributes('inert')).toBeUndefined()
  })

  it('makes its content inert once closed, so nothing out of sight takes focus', async () => {
    const wrapper = mount(AppCollapse, { props: { open: true }, slots: { default: '<button>Task</button>' } })

    await wrapper.setProps({ open: false })

    expect(wrapper.attributes('data-open')).toBe('false')
    expect(wrapper.attributes('inert')).toBeDefined()
    expect(wrapper.find('button').exists()).toBe(true)
  })
})
