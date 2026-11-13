import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import TimeSubNav from '@/features/time/components/TimeSubNav.vue'

describe('the Time sub-navigation', () => {
  it('links the report and the calendar and marks the one on screen', () => {
    const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { render: () => null } }] })
    const wrapper = mount(TimeSubNav, { props: { active: 'calendar' }, global: { plugins: [router] } })
    const tabs = wrapper.findAll('[role="tab"]')

    expect(tabs.map((tab) => [tab.text(), tab.attributes('href')])).toEqual([
      ['Report', '/report'],
      ['Calendar', '/calendar']
    ])
    expect(tabs.map((tab) => tab.attributes('aria-selected'))).toEqual(['false', 'true'])
  })
})
