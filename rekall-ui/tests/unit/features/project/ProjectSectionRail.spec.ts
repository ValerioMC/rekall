import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ProjectSectionRail from '@/features/project/components/ProjectSectionRail.vue'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { ProjectId, ProjectSectionId } from '@/model/branded'

function section(id: string, title: string, body = 'text'): ProjectSection {
  return {
    id: id as ProjectSectionId,
    projectId: 'p-1' as ProjectId,
    title,
    bodyMarkdown: body,
    position: 0,
    createdAt: '2026-10-05T10:00:00Z',
    updatedAt: '2026-10-05T10:00:00Z'
  }
}

function rail(props: Partial<InstanceType<typeof ProjectSectionRail>['$props']> = {}) {
  return mount(ProjectSectionRail, {
    props: { sections: [], selectedId: null, loading: false, creating: false, ...props },
    attachTo: document.body
  })
}

describe('ProjectSectionRail', () => {
  it('lists the sections, marking the one on screen and the empty ones', () => {
    const wrapper = rail({ sections: [section('s1', 'Deployment'), section('s2', 'Glossary', '')], selectedId: 's1' as ProjectSectionId })

    expect(wrapper.find('[data-testid="rail-section-s1"]').attributes('aria-current')).toBe('page')
    expect(wrapper.find('[data-testid="rail-section-s2"]').attributes('aria-current')).toBeUndefined()
    expect(wrapper.find('[data-testid="rail-section-s2"]').text()).toContain('Empty')
    wrapper.unmount()
  })

  it('tells a project with no section what the space is for', () => {
    const wrapper = rail()

    expect(wrapper.find('[data-testid="rail-empty"]').text()).toContain('deployment')
    wrapper.unmount()
  })

  it('emits the section picked', async () => {
    const wrapper = rail({ sections: [section('s1', 'Deployment')] })

    await wrapper.find('[data-testid="rail-section-s1"]').trigger('click')

    expect(wrapper.emitted('select')).toEqual([['s1']])
    wrapper.unmount()
  })

  it('creates a section from a title typed in the rail, trimmed, and closes the field', async () => {
    const wrapper = rail()

    await wrapper.find('[data-testid="rail-new-section"]').trigger('click')
    await wrapper.find('[data-testid="rail-new-title"]').setValue('  Deployment  ')
    await wrapper.find('[data-testid="rail-new-form"]').trigger('submit')

    expect(wrapper.emitted('create')).toEqual([['Deployment']])
    expect(wrapper.find('[data-testid="rail-new-form"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('ignores a blank title and closes on Escape', async () => {
    const wrapper = rail()

    await wrapper.find('[data-testid="rail-new-section"]').trigger('click')
    await wrapper.find('[data-testid="rail-new-title"]').setValue('   ')
    await wrapper.find('[data-testid="rail-new-form"]').trigger('submit')
    expect(wrapper.emitted('create')).toBeUndefined()

    await wrapper.find('[data-testid="rail-new-title"]').trigger('keydown', { key: 'Escape' })
    expect(wrapper.find('[data-testid="rail-new-form"]').exists()).toBe(false)
    wrapper.unmount()
  })
})
