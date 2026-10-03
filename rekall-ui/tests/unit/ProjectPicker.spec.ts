import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ProjectPicker from '@/components/console/ProjectPicker.vue'
import type { Project } from '@/model/catalog'
import type { CompanyId, ProjectId } from '@/model/branded'

function project(id: string, title: string, companyName: string, status: Project['status'] = 'ACTIVE'): Project {
  return {
    id: id as ProjectId,
    label: title.toLowerCase(),
    title,
    status,
    icon: 'folder',
    description: null,
    blueprintMarkdown: null,
    repoFolder: null,
    autoCommit: false,
    companyId: companyName as CompanyId,
    companyName,
    taskCount: 3,
    anchor: `project:${title.toLowerCase()}`,
    updatedAt: ''
  }
}

const projects = [
  project('p1', 'Vega', 'Acme'),
  project('p2', 'Beacon', 'Acme', 'PAUSED'),
  project('p3', 'Orbit', 'Globex')
]

function render(modelValue = 'p1') {
  return mount(ProjectPicker, { props: { modelValue, projects } })
}

describe('ProjectPicker', () => {
  it('describes the current project while closed', () => {
    const wrapper = render()

    expect(wrapper.find('[data-testid="project-picker-title"]').text()).toBe('Vega')
    expect(wrapper.text()).toContain('Acme')
    expect(wrapper.text()).toContain('project:vega')
    expect(wrapper.find('[data-testid="project-picker-search"]').exists()).toBe(false)
  })

  it('filters the list as the person types', async () => {
    const wrapper = render()
    await wrapper.find('[data-testid="project-picker-trigger"]').trigger('click')

    await wrapper.find('[data-testid="project-picker-search"]').setValue('globex')

    const options = wrapper.findAll('[data-testid="project-picker-option"]')
    expect(options).toHaveLength(1)
    expect(options[0]!.text()).toContain('Orbit')
  });

  it('says so when nothing matches', async () => {
    const wrapper = render()
    await wrapper.find('[data-testid="project-picker-trigger"]').trigger('click')

    await wrapper.find('[data-testid="project-picker-search"]').setValue('zzz')

    expect(wrapper.find('[data-testid="project-picker-empty"]').exists()).toBe(true)
  })

  it('picks with the arrow keys and Enter, then folds away', async () => {
    const wrapper = render()
    await wrapper.find('[data-testid="project-picker-trigger"]').trigger('click')
    const search = wrapper.find('[data-testid="project-picker-search"]')

    await search.trigger('keydown', { key: 'ArrowDown' })
    await search.trigger('keydown', { key: 'Enter' })

    expect(wrapper.emitted('update:modelValue')).toEqual([['p2']])
    expect(wrapper.find('[data-testid="project-picker-search"]').exists()).toBe(false)
  })

  it('picks with a click and reports open and close', async () => {
    const wrapper = render()
    await wrapper.find('[data-testid="project-picker-trigger"]').trigger('click')
    await wrapper.findAll('[data-testid="project-picker-option"]')[2]!.trigger('click')

    expect(wrapper.emitted('update:modelValue')).toEqual([['p3']])
    expect(wrapper.emitted('open-change')).toEqual([[true], [false]])
  })

  it('closes on Escape without picking', async () => {
    const wrapper = render()
    await wrapper.find('[data-testid="project-picker-trigger"]').trigger('click')

    await wrapper.find('[data-testid="project-picker-search"]').trigger('keydown', { key: 'Escape' })

    expect(wrapper.emitted('update:modelValue')).toBeUndefined()
    expect(wrapper.find('[data-testid="project-picker-search"]').exists()).toBe(false)
  })
})
