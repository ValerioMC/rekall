import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import ScopeSwitcher from '@/features/catalog/components/ScopeSwitcher.vue'
import { useConsoleStore } from '@/stores/console.store'
import { usePinnedProjectsStore } from '@/features/catalog/store/pinnedProjects.store'
import { useRecentProjectsStore } from '@/features/catalog/store/recentProjects.store'
import { useToastStore } from '@/stores/toast.store'
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

function render(companyId: CompanyId | null, projectId: ProjectId | null = null, projects: Project[] = catalog): VueWrapper {
  const store = useConsoleStore()
  store.companies = companies
  store.projects = projects
  wrapper = mount(ScopeSwitcher, { props: { companyId, projectId }, global: { plugins: [pinia] }, attachTo: document.body })
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
  it('reads Projects with its own glyph when no project is open', () => {
    const trigger = render(null).get('[data-testid="scope-trigger"]')

    expect(trigger.get('[data-testid="scope-trigger-title"]').text()).toBe('Projects')
    expect(trigger.find('[data-testid="scope-trigger-mark"]').exists()).toBe(false)
  })

  it("wears the company's logo when only the company is in scope", () => {
    const mark = render(acme).get('[data-testid="scope-trigger-mark"]')

    expect(mark.get('img').attributes('src')).toContain(`/api/companies/${acme}/logo-image`)
    expect(mark.find('[data-testid="scope-trigger-project-mark"]').exists()).toBe(false)
  })

  it('falls back to the initial for a company without a logo', () => {
    expect(render(globex).get('[data-testid="company-mark-initial"]').text()).toBe('G')
  })

  it("names the company and the project, with the project's mark on the company's", () => {
    const trigger = render(acme, vega).get('[data-testid="scope-trigger"]')

    expect(trigger.text()).toContain('acme')
    expect(trigger.get('[data-testid="scope-trigger-title"]').text()).toBe('Vega')
    expect(trigger.get('[data-testid="scope-trigger-project-mark"]').find('[data-testid="project-mark-glyph"]').exists()).toBe(true)
  })
})

describe('the switcher list', () => {
  it('opens with the search field focused and closes on Escape', async () => {
    const view = render(null)
    await open(view)

    expect(document.activeElement).toBe(view.get('[data-testid="switcher-search"]').element)
    await view.get('[data-testid="scope-popover"]').trigger('keydown', { key: 'Escape' })
    expect(view.find('[data-testid="scope-popover"]').exists()).toBe(false)
  })

  it('lists every company with its projects, each company a fold key', async () => {
    const view = render(acme, vega)
    await open(view)
    expect(view.findAll('[data-testid="scope-company"]')).toHaveLength(0)
    expect(view.findAll('[data-testid="scope-company-fold"]')).toHaveLength(2)
    expect(view.find('[data-testid="scope-all"]').exists()).toBe(false)
  })

  it("unfolds only the open project's company, and a fold key opens or closes any other", async () => {
    const view = render(acme, vega)
    await open(view)
    const titles = () => view.findAll('[data-testid="scope-project"]').map((row) => row.text().replace(/\d+$/, '').trim())
    expect(titles()).toEqual(['Atlas', 'Vega'])

    await view.findAll('[data-testid="scope-company-fold"]')[1]!.trigger('click')
    expect(titles()).toEqual(['Atlas', 'Vega', 'Lumen'])

    await view.findAll('[data-testid="scope-company-fold"]')[0]!.trigger('click')
    expect(titles()).toEqual(['Lumen'])
  })

  it('folds every company when nothing is in view but there are pins or recents to lead with, and unfolds them all when there are none', async () => {
    const bare = render(null)
    await open(bare)
    expect(bare.findAll('[data-testid="scope-project"]')).toHaveLength(3)
    bare.unmount()

    useRecentProjectsStore().remember(lumen)
    const led = render(null)
    await open(led)
    expect(led.findAll('[data-testid="scope-project"]')).toHaveLength(1)
  })

  it('unfolds every company while searching, so a match is never hidden', async () => {
    const view = render(acme, vega)
    await open(view)

    await view.get('[data-testid="switcher-search"]').setValue('lumen')

    expect(view.findAll('[data-testid="scope-project"]')).toHaveLength(1)
  })

  it('leaves an archived project out unless it is the one open', async () => {
    const archived = catalog.map((entry) => ({ ...entry, status: 'ARCHIVED' as const }))

    const widened = render(acme, null, archived)
    await open(widened)
    expect(widened.findAll('[data-testid="scope-project"]')).toHaveLength(0)
    widened.unmount()

    const open1 = render(acme, vega, archived)
    await open(open1)
    expect(open1.findAll('[data-testid="scope-project"]')).toHaveLength(1)
  })

  it('marks every company row with its logo or its initial and every project row with its icon', async () => {
    const view = render(null, null)
    await open(view)

    const marks = view.findAll('[data-testid="scope-company-fold"]').map((row) => row.find('[data-testid^="company-mark-"]').attributes('data-testid'))
    expect(marks).toEqual(['company-mark-image', 'company-mark-initial'])
    expect(view.get('[data-testid="scope-project"]').find('[data-testid^="project-mark-"]').exists()).toBe(true)
  })

  it('filters by what is typed and says so when nothing matches', async () => {
    const view = render(null)
    await open(view)

    await view.get('[data-testid="switcher-search"]').setValue('lum')
    expect(view.findAll('[data-testid="scope-project"]').map((row) => row.text())).toEqual([expect.stringContaining('Lumen')])

    await view.get('[data-testid="switcher-search"]').setValue('zzz')
    expect(view.get('[data-testid="switcher-empty"]').text()).toContain('zzz')
  })

  it('lists the recently opened projects first', async () => {
    const recent = useRecentProjectsStore()
    recent.remember(lumen)
    recent.remember(atlas)
    const view = render(null)
    await open(view)

    const titles = view.findAll('[data-testid="scope-project"]').map((row) => row.text())
    expect(titles.slice(0, 2).map((text) => text.replace(/\d+$/, '').trim())).toEqual(['Atlas', 'Lumen'])
    expect(titles).toHaveLength(2)

    await view.findAll('[data-testid="scope-company-fold"]')[0]!.trigger('click')
    expect(view.findAll('[data-testid="scope-project"]')).toHaveLength(4)
  })
})

describe('pinning in the switcher', () => {
  it('lists the pinned projects first, under their own heading, and not again under Recent', async () => {
    usePinnedProjectsStore().toggle(lumen)
    useRecentProjectsStore().remember(lumen)
    useRecentProjectsStore().remember(atlas)
    const view = render(null)
    await open(view)

    expect(view.get('[data-testid="scope-pinned-heading"]').text()).toBe('Pinned')
    const rows = view.findAll('[data-testid="scope-project"]').map((row) => row.text().replace(/\d+$/, '').trim())
    expect(rows.slice(0, 2)).toEqual(['Lumen', 'Atlas'])
  })

  it('pins and unpins from the star of a row without choosing the project', async () => {
    const view = render(acme, vega)
    await open(view)

    await view.findAll('[data-testid="scope-pin"]')[0]!.trigger('click')

    expect(usePinnedProjectsStore().ids).toEqual([atlas])
    expect(emitted(view)).toEqual([])
    expect(view.findAll('[data-testid="scope-pin"]')[0]!.attributes('aria-pressed')).toBe('true')
  })

  it('says so, and pins nothing, when three projects are already pinned', async () => {
    const pins = usePinnedProjectsStore()
    ;[vega, atlas, lumen].forEach((id) => pins.toggle(id))
    const other = project('p4' as ProjectId, 'Orion', acme, 'acme')
    const view = render(acme, other.id, [...catalog, other])
    await open(view)

    const orionStar = view.findAll('[data-testid="scope-project-row"]').find((row) => row.text().includes('Orion'))!
    await orionStar.get('[data-testid="scope-pin"]').trigger('click')

    expect(pins.ids).toEqual([vega, atlas, lumen])
    expect(useToastStore().toasts.at(-1)?.message).toContain('at most 3')
  })
})

describe('the header variant', () => {
  it('wears the project in view and its title, and a Projects label when none is', () => {
    const open = render(acme, vega)
    expect(open.get('[data-testid="scope-trigger"]').attributes('aria-label')).toBe('Switch project')
    open.unmount()

    const none = mount(ScopeSwitcher, { props: { companyId: null, projectId: null, variant: 'header' }, global: { plugins: [pinia] } })
    expect(none.get('[data-testid="scope-trigger-title"]').text()).toBe('Projects')
    none.unmount()
  })
})

describe('choosing in the switcher', () => {
  it('hands back the project and its company, and remembers the project', async () => {
    const view = render(null, null)
    await open(view)
    await view.findAll('[data-testid="scope-project"]')[0]!.trigger('click')

    expect(emitted(view)).toEqual([{ companyId: acme, projectId: atlas }])
    expect(useRecentProjectsStore().ids).toEqual([atlas])
  })

  it('starts on the open project and wraps to the first row with the arrow and Enter', async () => {
    const view = render(acme, vega)
    await open(view)
    const search = view.get('[data-testid="switcher-search"]')

    await search.trigger('keydown', { key: 'ArrowDown' })
    await search.trigger('keydown', { key: 'Enter' })

    expect(emitted(view)).toEqual([{ companyId: acme, projectId: atlas }])
  })
})

describe('creating from the switcher', () => {
  it('offers a company and a project, and opens the record dialog for either', async () => {
    const view = render(null, null)
    await open(view)

    expect(view.find('[data-testid="new-project"]').exists()).toBe(true)
    await view.get('[data-testid="new-company"]').trigger('click')

    expect(view.find('[data-testid="record-dialog"]').exists()).toBe(true)
    expect(view.find('[data-testid="record-label"]').exists()).toBe(false)
  })
})
