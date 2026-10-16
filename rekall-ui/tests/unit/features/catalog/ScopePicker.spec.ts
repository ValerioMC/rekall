import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { mount, type VueWrapper } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import ScopePicker from '@/features/catalog/components/ScopePicker.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Company, Project } from '@/features/catalog/model/catalog'
import type { CompanyId, ProjectId } from '@/model/branded'

/** The first column's scope picker: which marks its trigger and its company rows wear. */
const acme = 'c1' as CompanyId
const globex = 'c2' as CompanyId
const vega = 'p1' as ProjectId

const companies: Company[] = [
  { id: acme, name: 'acme', description: null, logoImageVersion: 'v7', projectCount: 1, taskCount: 0, updatedAt: '' },
  { id: globex, name: 'globex', description: null, logoImageVersion: null, projectCount: 0, taskCount: 0, updatedAt: '' }
]

const projects: Project[] = [
  { id: vega, label: 'vega', title: 'Vega', status: 'ACTIVE', icon: 'rocket', iconImageVersion: null, description: null, blueprintMarkdown: null,
    repoFolder: null, autoCommit: false, companyId: acme, companyName: 'acme', taskCount: 0, anchor: 'project:vega', updatedAt: '' }
]

let pinia: Pinia

function render(company: CompanyId | null, project: ProjectId | null = null, catalog: Project[] = projects): VueWrapper {
  const store = useConsoleStore()
  store.companies = companies
  store.projects = catalog
  store.setScope(company, project)
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/', component: { template: '<div />' } }] })
  return mount(ScopePicker, { global: { plugins: [pinia, router] } })
}

beforeEach(() => {
  pinia = createPinia()
  setActivePinia(pinia)
})

describe('ScopePicker trigger', () => {
  it('keeps its own glyph for all work', () => {
    const trigger = render(null).get('[data-testid="scope-trigger"]')

    expect(trigger.find('[data-testid="scope-trigger-mark"]').exists()).toBe(false)
  })

  it("wears the company's logo when the company is in scope", () => {
    const mark = render(acme).get('[data-testid="scope-trigger-mark"]')

    expect(mark.get('img').attributes('src')).toContain(`/api/companies/${acme}/logo-image`)
    expect(mark.find('[data-testid="scope-trigger-project-mark"]').exists()).toBe(false)
  })

  it("wears the owning company's logo with the project's mark on it when a project is in scope", () => {
    const mark = render(acme, vega).get('[data-testid="scope-trigger-mark"]')

    expect(mark.find('[data-testid="company-mark-image"]').exists()).toBe(true)
    expect(mark.get('[data-testid="scope-trigger-project-mark"]').find('[data-testid="project-mark-glyph"]').exists()).toBe(true)
  })

  it('falls back to the initial for a company without a logo', () => {
    const mark = render(globex).get('[data-testid="scope-trigger-mark"]')

    expect(mark.get('[data-testid="company-mark-initial"]').text()).toBe('G')
  })
})

describe('ScopePicker list', () => {
  it('leaves an archived project out unless it is the one in scope', async () => {
    const archived = projects.map((project) => ({ ...project, status: 'ARCHIVED' as const }))

    const widened = render(acme, null, archived)
    await widened.get('[data-testid="scope-trigger"]').trigger('click')
    expect(widened.findAll('[data-testid="scope-project"]')).toHaveLength(0)

    const scoped = render(acme, vega, archived)
    await scoped.get('[data-testid="scope-trigger"]').trigger('click')
    expect(scoped.findAll('[data-testid="scope-project"]')).toHaveLength(1)
  })

  it('marks every company row, with its logo or its initial', async () => {
    const picker = render(null)
    await picker.get('[data-testid="scope-trigger"]').trigger('click')

    const rows = picker.findAll('[data-testid="scope-company"]')
    expect(rows.map((row) => row.find('[data-testid^="company-mark-"]').attributes('data-testid'))).toEqual([
      'company-mark-image',
      'company-mark-initial'
    ])
  })
})
