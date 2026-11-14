import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import ScopeSwitcher from '@/features/catalog/components/ScopeSwitcher.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useRecentProjectsStore } from '@/features/catalog/store/recentProjects.store'
import { installStorage } from '../../support/memoryStorage'
import type { Company, Project } from '@/features/catalog/model/catalog'
import type { ScopeTarget } from '@/features/catalog/model/scope-target'
import type { CompanyId, ProjectId } from '@/model/branded'

const acme = 'c1' as CompanyId
const globex = 'c2' as CompanyId
const vega = 'p1' as ProjectId
const atlas = 'p2' as ProjectId
const lumen = 'p3' as ProjectId

const companies: Company[] = [
  { id: acme, name: 'acme', description: null, logoImageVersion: 'v7', projectCount: 2, taskCount: 5, updatedAt: '' },
  { id: globex, name: 'globex', description: null, logoImageVersion: null, projectCount: 1, taskCount: 2, updatedAt: '' }
]

function project(id: ProjectId, title: string, companyId: CompanyId, companyName: string, status: Project['status'] = 'ACTIVE'): Project {
  return {
    id, label: title.toLowerCase(), taskKey: 'PR', title, status, icon: 'rocket', iconImageVersion: null, description: null,
    blueprintMarkdown: null, repoFolder: null, autoCommit: false, mainBranch: null, pullRequestFlow: false,
    companyId, companyName, taskCount: 3, anchor: `project:${title.toLowerCase()}`, updatedAt: ''
  }
}

const catalog: Project[] = [project(vega, 'Vega', acme, 'acme'), project(atlas, 'Atlas', acme, 'acme'), project(lumen, 'Lumen', globex, 'globex')]

let pinia: Pinia
let wrapper: VueWrapper | null = null

function render(mode: 'work' | 'project', companyId: CompanyId | null, projectId: ProjectId | null = null, projects: Project[] = catalog): VueWrapper {
  const store = useConsoleStore()
  store.companies = companies
  store.projects = projects
  wrapper = mount(ScopeSwitcher, { props: { mode, companyId, projectId }, global: { plugins: [pinia] }, attachTo: document.body })
  return wrapper
}

async function open(view: VueWrapper): Promise<void> {
  await view.get('[data-testid="scope-trigger"]').trigger('click')
  await flushPromises()
}

function emitted(view: VueWrapper): ScopeTarget[] {
  return (view.emitted('select') ?? []).map((args) => args[0] as ScopeTarget)
}

beforeEach(() => {
  installStorage()
  pinia = createPinia()
  setActivePinia(pinia)
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  delete (window as { localStorage?: unknown }).localStorage
})

describe('the switcher trigger', () => {
  it('reads All work with its own glyph when nothing is narrowed', () => {
    const trigger = render('work', null).get('[data-testid="scope-trigger"]')

    expect(trigger.get('[data-testid="scope-trigger-title"]').text()).toBe('All work')
    expect(trigger.find('[data-testid="scope-trigger-mark"]').exists()).toBe(false)
  })

  it("wears the company's logo when only the company is in scope", () => {
    const mark = render('work', acme).get('[data-testid="scope-trigger-mark"]')

    expect(mark.get('img').attributes('src')).toContain(`/api/companies/${acme}/logo-image`)
    expect(mark.find('[data-testid="scope-trigger-project-mark"]').exists()).toBe(false)
  })

  it('falls back to the initial for a company without a logo', () => {
    expect(render('work', globex).get('[data-testid="company-mark-initial"]').text()).toBe('G')
  })

  it("names the company and the project, with the project's mark on the company's", () => {
    const trigger = render('project', acme, vega).get('[data-testid="scope-trigger"]')

    expect(trigger.text()).toContain('acme')
    expect(trigger.get('[data-testid="scope-trigger-title"]').text()).toBe('Vega')
    expect(trigger.get('[data-testid="scope-trigger-project-mark"]').find('[data-testid="project-mark-glyph"]').exists()).toBe(true)
  })
})

describe('the switcher list', () => {
  it('opens with the search field focused and closes on Escape', async () => {
    const view = render('work', null)
    await open(view)

    expect(document.activeElement).toBe(view.get('[data-testid="switcher-search"]').element)
    await view.get('[data-testid="scope-popover"]').trigger('keydown', { key: 'Escape' })
    expect(view.find('[data-testid="scope-popover"]').exists()).toBe(false)
  })

  it('lists every company with its projects, and All work first on Work only', async () => {
    const work = render('work', null)
    await open(work)
    expect(work.find('[data-testid="scope-all"]').exists()).toBe(true)
    expect(work.findAll('[data-testid="scope-company"]')).toHaveLength(2)
    expect(work.findAll('[data-testid="scope-project"]')).toHaveLength(3)
    work.unmount()

    const page = render('project', acme, vega)
    await open(page)
    expect(page.find('[data-testid="scope-all"]').exists()).toBe(false)
    expect(page.findAll('[data-testid="scope-company"]')).toHaveLength(0)
    expect(page.findAll('[data-testid="scope-project"]')).toHaveLength(3)
  })

  it('leaves an archived project out unless it is the one open', async () => {
    const archived = catalog.map((entry) => ({ ...entry, status: 'ARCHIVED' as const }))

    const widened = render('work', acme, null, archived)
    await open(widened)
    expect(widened.findAll('[data-testid="scope-project"]')).toHaveLength(0)
    widened.unmount()

    const open1 = render('project', acme, vega, archived)
    await open(open1)
    expect(open1.findAll('[data-testid="scope-project"]')).toHaveLength(1)
  })

  it('marks every company row with its logo or its initial and every project row with its icon', async () => {
    const view = render('work', null)
    await open(view)

    const rows = view.findAll('[data-testid="scope-company"]')
    expect(rows.map((row) => row.find('[data-testid^="company-mark-"]').attributes('data-testid'))).toEqual([
      'company-mark-image',
      'company-mark-initial'
    ])
    expect(view.get('[data-testid="scope-project"]').find('[data-testid^="project-mark-"]').exists()).toBe(true)
  })

  it('filters by what is typed and says so when nothing matches', async () => {
    const view = render('work', null)
    await open(view)

    await view.get('[data-testid="switcher-search"]').setValue('lum')
    expect(view.findAll('[data-testid="scope-project"]').map((row) => row.text())).toEqual([expect.stringContaining('Lumen')])
    expect(view.find('[data-testid="scope-all"]').exists()).toBe(false)

    await view.get('[data-testid="switcher-search"]').setValue('zzz')
    expect(view.get('[data-testid="switcher-empty"]').text()).toContain('zzz')
  })

  it('lists the recently opened projects first', async () => {
    const recent = useRecentProjectsStore()
    recent.remember(lumen)
    recent.remember(atlas)
    const view = render('work', null)
    await open(view)

    const titles = view.findAll('[data-testid="scope-project"]').map((row) => row.text())
    expect(titles.slice(0, 2).map((text) => text.replace(/\d+$/, '').trim())).toEqual(['Atlas', 'Lumen'])
    expect(titles).toHaveLength(5)
  })
})

describe('choosing in the switcher', () => {
  it('hands back all work, a company or a project, and remembers the project', async () => {
    const view = render('work', null)
    await open(view)
    await view.get('[data-testid="scope-all"]').trigger('click')
    await open(view)
    await view.findAll('[data-testid="scope-company"]')[1]!.trigger('click')
    await open(view)
    await view.findAll('[data-testid="scope-project"]')[0]!.trigger('click')

    expect(emitted(view)).toEqual([
      { kind: 'all' },
      { kind: 'company', companyId: globex },
      { kind: 'project', companyId: acme, projectId: atlas }
    ])
    expect(useRecentProjectsStore().ids).toEqual([atlas])
  })

  it('starts on the open project and picks the next row with the arrow and Enter', async () => {
    const view = render('project', acme, vega)
    await open(view)
    const search = view.get('[data-testid="switcher-search"]')

    await search.trigger('keydown', { key: 'ArrowDown' })
    await search.trigger('keydown', { key: 'Enter' })

    expect(emitted(view)).toEqual([{ kind: 'project', companyId: globex, projectId: lumen }])
  })

  it('opens the page of a project from its row, on Work', async () => {
    const view = render('work', null)
    await open(view)

    await view.findAll('[data-testid="scope-open-page"]')[0]!.trigger('click')

    expect(view.emitted('open-page')).toEqual([[atlas]])
    expect(view.find('[data-testid="scope-popover"]').exists()).toBe(false)
  })

  it('offers no page link on a project page, where the page is already open', async () => {
    const view = render('project', acme, vega)
    await open(view)

    expect(view.find('[data-testid="scope-open-page"]').exists()).toBe(false)
  })
})
