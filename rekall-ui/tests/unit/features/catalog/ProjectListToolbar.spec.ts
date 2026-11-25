import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ProjectListToolbar from '@/features/catalog/components/ProjectListToolbar.vue'
import type { Company } from '@/features/catalog/model/catalog'

const OPTIONS = [
  { value: 'ALL', label: 'All', count: 3 },
  { value: 'ACTIVE', label: 'Active', count: 2 }
] as const

const ACME = { id: 'c1', name: 'Acme', logoImageVersion: null } as unknown as Company

function render(props: Partial<InstanceType<typeof ProjectListToolbar>['$props']> = {}) {
  return mount(ProjectListToolbar, {
    props: { modelValue: 'ALL', options: OPTIONS, company: null, companyPinned: false, total: 3, reviewing: 0, ...props }
  })
}

describe('the project list toolbar', () => {
  it('counts the projects and what waits for review', () => {
    expect(render({ total: 1 }).text()).toContain('1 project')
    expect(render({ total: 3, reviewing: 2 }).text()).toMatch(/3\s*projects\s*·\s*2\s*to review/)
  })

  it('shows the pinned company and clears it', async () => {
    const toolbar = render({ company: ACME, companyPinned: true })

    expect(toolbar.get('[data-testid="company-filter"]').text()).toContain('Acme')
    await toolbar.get('[data-testid="company-filter"]').trigger('click')
    expect(toolbar.emitted('clearCompany')).toHaveLength(1)
  })

  it('shows no company chip when the list is not pinned', () => {
    expect(render().find('[data-testid="company-filter"]').exists()).toBe(false)
  })

  it('changes the status through the segments', async () => {
    const toolbar = render()

    await toolbar.get('[data-testid="segment-ACTIVE"]').trigger('click')

    expect(toolbar.emitted('update:modelValue')).toEqual([['ACTIVE']])
  })
})
