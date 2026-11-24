import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ProjectSectionRail from '@/features/project/components/ProjectSectionRail.vue'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { SectionContext } from '@/features/project/model/section-context'
import type { ProjectId, ProjectSectionId, SectionContextId } from '@/model/branded'

const OPS = 'ops' as SectionContextId
const TEAM = 'team' as SectionContextId

function context(id: SectionContextId, title: string, position: number): SectionContext {
  return { id, projectId: 'p-1' as ProjectId, title, position, createdAt: '2026-10-05T10:00:00Z', updatedAt: '2026-10-05T10:00:00Z' }
}

function section(id: string, title: string, contextId: SectionContextId = OPS, body = 'text'): ProjectSection {
  return {
    id: id as ProjectSectionId,
    projectId: 'p-1' as ProjectId,
    contextId,
    title,
    bodyMarkdown: body,
    position: 0,
    createdAt: '2026-10-05T10:00:00Z',
    updatedAt: '2026-10-05T10:00:00Z'
  }
}

const CONTEXTS = [context(OPS, 'Operations', 0), context(TEAM, 'Team', 1)]

function rail(props: Partial<InstanceType<typeof ProjectSectionRail>['$props']> = {}) {
  return mount(ProjectSectionRail, {
    props: { contexts: [], sections: [], selectedId: null, loading: false, creating: false, autoconfiguring: false, reorganizing: false, ...props },
    attachTo: document.body
  })
}

function dragData() {
  return { dataTransfer: { setData: vi.fn(), effectAllowed: '' } }
}

describe('ProjectSectionRail', () => {
  it('groups the pages under their contexts with a count, marking the one on screen and the empty ones', () => {
    const wrapper = rail({
      contexts: CONTEXTS,
      sections: [section('s1', 'Deployment'), section('s2', 'Glossary', TEAM, '')],
      selectedId: 's1' as ProjectSectionId
    })

    expect(wrapper.find('[data-testid="rail-context-ops"]').text()).toContain('Operations')
    expect(wrapper.find('[data-testid="rail-context-ops"]').find('[data-testid="rail-section-s1"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="rail-context-team"]').find('[data-testid="rail-section-s2"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="rail-section-s1"]').attributes('aria-current')).toBe('page')
    expect(wrapper.find('[data-testid="rail-section-s2"]').attributes('aria-current')).toBeUndefined()
    expect(wrapper.find('[data-testid="rail-section-s2"]').text()).toContain('Empty')
    wrapper.unmount()
  })

  it('shows how big a page is and asks to import a file', async () => {
    const wrapper = rail({ contexts: CONTEXTS, sections: [section('s1', 'Reference', OPS, 'x'.repeat(62_410))] })

    expect(wrapper.find('[data-testid="rail-section-s1"]').text()).toContain('62k')
    await wrapper.find('[data-testid="rail-import"]').trigger('click')

    expect(wrapper.emitted('import')).toHaveLength(1)
    wrapper.unmount()
  })

  it('asks for an autoconfiguration, and not again while one is starting', async () => {
    const wrapper = rail()

    await wrapper.find('[data-testid="rail-autoconfigure"]').trigger('click')
    await wrapper.setProps({ autoconfiguring: true })

    expect(wrapper.emitted('autoconfigure')).toHaveLength(1)
    expect(wrapper.find('[data-testid="rail-autoconfigure"]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('asks for a reorganization once there are two pages to group, and not again while one is starting', async () => {
    const wrapper = rail({ contexts: CONTEXTS, sections: [section('s1', 'Deployment')] })
    expect(wrapper.find('[data-testid="rail-reorganize"]').attributes('disabled')).toBeDefined()

    await wrapper.setProps({ sections: [section('s1', 'Deployment'), section('s2', 'Glossary')] })
    await wrapper.find('[data-testid="rail-reorganize"]').trigger('click')
    await wrapper.setProps({ reorganizing: true })

    expect(wrapper.emitted('reorganize')).toHaveLength(1)
    expect(wrapper.find('[data-testid="rail-reorganize"]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('tells a project with no page what the space is for', () => {
    const wrapper = rail()

    expect(wrapper.find('[data-testid="rail-empty"]').text()).toContain('deployment')
    wrapper.unmount()
  })

  it('emits the page picked', async () => {
    const wrapper = rail({ contexts: CONTEXTS, sections: [section('s1', 'Deployment')] })

    await wrapper.find('[data-testid="rail-section-s1"]').trigger('click')

    expect(wrapper.emitted('select')).toEqual([['s1']])
    wrapper.unmount()
  })

  it('starts a page in the context of the one on screen, trimmed, and closes the field', async () => {
    const wrapper = rail({ contexts: CONTEXTS, sections: [section('s1', 'Deployment'), section('s2', 'Glossary', TEAM)], selectedId: 's2' as ProjectSectionId })

    await wrapper.find('[data-testid="rail-new-section"]').trigger('click')
    await wrapper.find('[data-testid="rail-context-page-title-team"]').setValue('  Roles  ')
    await wrapper.find('[data-testid="rail-context-page-form-team"]').trigger('submit')

    expect(wrapper.emitted('create')).toEqual([['Roles', 'team']])
    expect(wrapper.find('[data-testid="rail-context-page-form-team"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('starts a page in a context from the key on its header', async () => {
    const wrapper = rail({ contexts: CONTEXTS, sections: [section('s1', 'Deployment')], selectedId: 's1' as ProjectSectionId })

    await wrapper.find('[data-testid="rail-context-new-page-team"]').trigger('click')
    await wrapper.find('[data-testid="rail-context-page-title-team"]').setValue('Roles')
    await wrapper.find('[data-testid="rail-context-page-form-team"]').trigger('submit')

    expect(wrapper.emitted('create')).toEqual([['Roles', 'team']])
    wrapper.unmount()
  })

  it('starts the first page of a project with no context without naming one', async () => {
    const wrapper = rail()

    await wrapper.find('[data-testid="rail-new-section"]').trigger('click')
    await wrapper.find('[data-testid="rail-new-title"]').setValue('  Deployment  ')
    await wrapper.find('[data-testid="rail-new-form"]').trigger('submit')

    expect(wrapper.emitted('create')).toEqual([['Deployment', null]])
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

  it('adds a context from a title typed in the rail', async () => {
    const wrapper = rail({ contexts: CONTEXTS })

    await wrapper.find('[data-testid="rail-new-context"]').trigger('click')
    await wrapper.find('[data-testid="rail-new-context-title"]').setValue('  Onboarding ')
    await wrapper.find('[data-testid="rail-new-context-form"]').trigger('submit')

    expect(wrapper.emitted('createContext')).toEqual([['Onboarding']])
    expect(wrapper.find('[data-testid="rail-new-context-form"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('folds a context away and opens it again', async () => {
    const wrapper = rail({ contexts: CONTEXTS, sections: [section('s1', 'Deployment')] })

    await wrapper.find('[data-testid="rail-context-toggle-ops"]').trigger('click')
    expect(wrapper.find('[data-testid="rail-section-s1"]').exists()).toBe(false)
    await wrapper.find('[data-testid="rail-context-toggle-ops"]').trigger('click')

    expect(wrapper.find('[data-testid="rail-section-s1"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('says an empty context holds no page', () => {
    const wrapper = rail({ contexts: CONTEXTS, sections: [section('s1', 'Deployment')] })

    expect(wrapper.find('[data-testid="rail-context-empty-team"]').text()).toContain('No pages')
    expect(wrapper.find('[data-testid="rail-context-empty-ops"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('renames a context in place and ignores a title it already has', async () => {
    const wrapper = rail({ contexts: CONTEXTS })

    await wrapper.find('[data-testid="rail-context-options-ops"]').trigger('click')
    await wrapper.findAll('[data-testid="popover-menu-item"]').find((item) => item.text() === 'Rename')!.trigger('click')
    await wrapper.find('[data-testid="rail-context-rename-ops"]').setValue('  Run  ')
    await wrapper.find('[data-testid="rail-context-rename-ops"]').trigger('blur')

    expect(wrapper.emitted('renameContext')).toEqual([['ops', 'Run']])
    wrapper.unmount()
  })

  it('moves a context only where it has room, and deletes one only once it holds no page', async () => {
    const wrapper = rail({ contexts: CONTEXTS, sections: [section('s1', 'Deployment')] })

    await wrapper.find('[data-testid="rail-context-options-ops"]').trigger('click')
    const first = wrapper.findAll('[data-testid="popover-menu-item"]')
    expect(first.find((item) => item.text() === 'Move context up')!.attributes('disabled')).toBeDefined()
    expect(first.find((item) => item.text().startsWith('Delete context'))!.attributes('disabled')).toBeDefined()
    expect(first.find((item) => item.text().startsWith('Delete context'))!.text()).toContain('Move its 1 page out first')
    await first.find((item) => item.text() === 'Move context down')!.trigger('click')

    await wrapper.find('[data-testid="rail-context-options-team"]').trigger('click')
    const second = wrapper.findAll('[data-testid="popover-menu-item"]')
    await second.find((item) => item.text() === 'Delete context')!.trigger('click')

    expect(wrapper.emitted('moveContext')).toEqual([['ops', 'down']])
    expect(wrapper.emitted('deleteContext')).toEqual([['team']])
    wrapper.unmount()
  })

  it('moves a page into the context it is dropped on, last, or before the page it is dropped on', async () => {
    const wrapper = rail({ contexts: CONTEXTS, sections: [section('s1', 'Deployment'), section('s2', 'Glossary', TEAM), section('s3', 'Roles', TEAM)] })

    await wrapper.find('[data-testid="rail-section-s1"]').trigger('dragstart', dragData())
    await wrapper.find('[data-testid="rail-context-toggle-team"]').trigger('dragover')
    await wrapper.find('[data-testid="rail-context-toggle-team"]').trigger('drop')

    await wrapper.find('[data-testid="rail-section-s1"]').trigger('dragstart', dragData())
    await wrapper.find('[data-testid="rail-section-s3"]').trigger('dragover')
    await wrapper.find('[data-testid="rail-section-s3"]').trigger('drop')

    expect(wrapper.emitted('movePage')).toEqual([
      ['s1', 'team', null],
      ['s1', 'team', 's3']
    ])
    wrapper.unmount()
  })
})
