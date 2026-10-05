import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ProjectDocsRail from '@/components/project/ProjectDocsRail.vue'
import type { ProjectSection } from '@/model/project-section'
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

function rail(props: Partial<InstanceType<typeof ProjectDocsRail>['$props']> = {}) {
  return mount(ProjectDocsRail, {
    props: { sections: [], selected: 'blueprint', loading: false, creating: false, ...props },
    attachTo: document.body
  })
}

describe('ProjectDocsRail', () => {
  it('pins the blueprint and lists the sections under it, marking the one on screen', () => {
    const wrapper = rail({ sections: [section('s1', 'Deployment'), section('s2', 'Glossary', '')], selected: 's1' as ProjectSectionId })

    expect(wrapper.find('[data-testid="rail-blueprint"]').text()).toContain('Loaded by every /rk')
    expect(wrapper.find('[data-testid="rail-section-s1"]').attributes('aria-current')).toBe('page')
    expect(wrapper.find('[data-testid="rail-blueprint"]').attributes('aria-current')).toBeUndefined()
    expect(wrapper.find('[data-testid="rail-section-s2"]').text()).toContain('Empty')
    wrapper.unmount()
  })

  it('tells a project with no section what the space is for', () => {
    const wrapper = rail()

    expect(wrapper.find('[data-testid="rail-empty"]').text()).toContain('deployment')
    wrapper.unmount()
  })

  it('emits what was picked', async () => {
    const wrapper = rail({ sections: [section('s1', 'Deployment')] })

    await wrapper.find('[data-testid="rail-section-s1"]').trigger('click')
    await wrapper.find('[data-testid="rail-blueprint"]').trigger('click')

    expect(wrapper.emitted('select')).toEqual([['s1'], ['blueprint']])
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
