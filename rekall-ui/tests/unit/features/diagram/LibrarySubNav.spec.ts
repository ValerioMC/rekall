import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import LibrarySubNav from '@/features/diagram/components/LibrarySubNav.vue'

describe('the Library sub-navigation', () => {
  it('links the diagrams and the Claude files and marks the one on screen', () => {
    const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { render: () => null } }] })
    const wrapper = mount(LibrarySubNav, { props: { active: 'diagrams' }, global: { plugins: [router] } })
    const tabs = wrapper.findAll('[role="tab"]')

    expect(tabs.map((tab) => [tab.text(), tab.attributes('href')])).toEqual([
      ['Diagrams', '/diagrams'],
      ['Claude files', '/claude']
    ])
    expect(tabs.map((tab) => tab.attributes('aria-selected'))).toEqual(['true', 'false'])
  })
})
