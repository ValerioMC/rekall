import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ProjectCompositionLegend from '@/features/catalog/components/ProjectCompositionLegend.vue'

describe('the composition legend', () => {
  it('names each part that has tasks with its count', () => {
    const legend = mount(ProjectCompositionLegend, { props: { composition: { filed: 4, review: 2, active: 0, todo: 1 } } })

    expect(legend.text()).toMatch(/4\s*filed/)
    expect(legend.text()).toMatch(/2\s*to review/)
    expect(legend.text()).toMatch(/1\s*to do/)
    expect(legend.text()).not.toContain('in progress')
  })

  it('says there are no tasks when every part is empty', () => {
    const legend = mount(ProjectCompositionLegend, { props: { composition: { filed: 0, review: 0, active: 0, todo: 0 } } })

    expect(legend.text()).toBe('No tasks')
  })
})
