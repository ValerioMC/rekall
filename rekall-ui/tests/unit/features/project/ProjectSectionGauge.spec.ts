import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ProjectSectionGauge from '@/features/project/components/ProjectSectionGauge.vue'

describe('ProjectSectionGauge', () => {
  it('states the size, the cost of reading it and that no context carries it', () => {
    const wrapper = mount(ProjectSectionGauge, { props: { characters: 62_410 } })

    expect(wrapper.text()).toContain('62,410')
    expect(wrapper.text()).toContain('of 100,000 characters')
    expect(wrapper.text()).toContain('≈ 15.6k tokens when read')
    expect(wrapper.find('[data-testid="section-gauge-context"]').text()).toContain('Not in context')
    expect(wrapper.find('[data-testid="section-gauge-over"]').exists()).toBe(false)
  })

  it('warns that a section over the limit will not save', () => {
    const wrapper = mount(ProjectSectionGauge, { props: { characters: 100_001 } })

    expect(wrapper.find('[data-testid="section-gauge-over"]').text()).toContain('will not save')
    expect(wrapper.find('[data-testid="section-gauge-context"]').exists()).toBe(false)
  })
})
