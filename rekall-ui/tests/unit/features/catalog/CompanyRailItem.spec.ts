import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import CompanyRailItem from '@/features/catalog/components/CompanyRailItem.vue'
import type { Company } from '@/features/catalog/model/catalog'

const ACME = { id: 'c1', name: 'Acme', projectCount: 3, taskCount: 12, logoImageVersion: null } as unknown as Company
const QUIET = { filed: 0, review: 0, active: 0, todo: 0 }

function render(props: Partial<InstanceType<typeof CompanyRailItem>['$props']> = {}) {
  return mount(CompanyRailItem, { props: { company: ACME, composition: QUIET, reviewing: 0, selected: false, ...props } })
}

describe('a company in the list', () => {
  it('shows its size and what waits for review', () => {
    const item = render({ composition: { filed: 5, review: 2, active: 0, todo: 5 }, reviewing: 2 })

    expect(item.text()).toContain('3 projects')
    expect(item.text()).toContain('12 tasks')
    expect(item.get('[data-testid="company-needs"]').text()).toBe('2 to review')
  })

  it('leaves out the review count when nothing waits', () => {
    expect(render().find('[data-testid="company-needs"]').exists()).toBe(false)
  })

  it('marks the selected company and selects on click', async () => {
    const item = render({ selected: true })

    expect(item.attributes('aria-current')).toBe('true')
    await item.trigger('click')
    expect(item.emitted('select')).toHaveLength(1)
  })
})
