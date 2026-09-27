import { beforeEach, describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import { defineComponent } from 'vue'
import AppNavGroup from '@/components/console/AppNavGroup.vue'
import { NAV_ENTRIES, activeScreenOf, type NavGroup } from '@/model/nav-tab'

const Blank = defineComponent({ template: '<div />' })

const CATALOG = NAV_ENTRIES.flatMap((entry) => (entry.kind === 'group' ? [entry.group] : [])).find(
  (group) => group.label === 'Catalog'
) as NavGroup

function routerAt(): Router {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', name: 'console', component: Blank },
      { path: '/projects', name: 'projects', component: Blank },
      { path: '/projects/:id', name: 'project-detail', component: Blank },
      { path: '/companies', name: 'companies', component: Blank }
    ]
  })
}

async function mountGroup(path: string) {
  const router = routerAt()
  await router.push(path)
  await router.isReady()
  const wrapper = mount(AppNavGroup, {
    props: { group: CATALOG },
    global: { plugins: [router] },
    attachTo: document.body
  })
  return { wrapper, router }
}

function visibleLabel(wrapper: Awaited<ReturnType<typeof mountGroup>>['wrapper']): string {
  return wrapper
    .findAll('[data-testid="nav-group-trigger"] .grid > span')
    .filter((label) => !label.classes().includes('invisible'))
    .map((label) => label.text())
    .join()
}

describe('a grouped switcher slot', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
  })

  it('wears the group name away from its screens and the member name on one of them', async () => {
    expect(visibleLabel((await mountGroup('/')).wrapper)).toBe('Catalog')
    expect(visibleLabel((await mountGroup('/projects/p-1')).wrapper)).toBe('Projects')
    expect(visibleLabel((await mountGroup('/companies')).wrapper)).toBe('Companies')
  })

  it('stacks every candidate name so its width never depends on which one shows', async () => {
    const { wrapper } = await mountGroup('/')
    const labels = wrapper
      .findAll('[data-testid="nav-group-trigger"] .grid > span')
      .map((label) => label.text())
    expect(labels).toEqual(['Catalog', 'Projects', 'Companies'])
  })

  it('opens its members as a menu, marks the one on screen, and closes on navigation', async () => {
    const { wrapper, router } = await mountGroup('/companies')

    await wrapper.get('[data-testid="nav-group-trigger"]').trigger('click')
    const items = wrapper.findAll('[data-testid="nav-group-item"]')
    expect(items.map((item) => item.attributes('aria-current'))).toEqual([undefined, 'page'])

    await router.push('/projects')
    await flushPromises()
    expect(wrapper.find('[data-testid="nav-group-menu"]').exists()).toBe(false)
  })

  it('closes on Escape and hands focus back to the slot', async () => {
    const { wrapper } = await mountGroup('/')
    const trigger = wrapper.get('[data-testid="nav-group-trigger"]')

    await trigger.trigger('keydown', { key: 'ArrowDown' })
    await flushPromises()
    expect(document.activeElement?.getAttribute('data-testid')).toBe('nav-group-item')

    await wrapper.get('[data-testid="nav-group-menu"]').trigger('keydown', { key: 'Escape' })
    expect(wrapper.find('[data-testid="nav-group-menu"]').exists()).toBe(false)
    expect(document.activeElement).toBe(trigger.element)
  })
})

describe('the active member of a group', () => {
  it('counts a project page as the Projects screen', () => {
    expect(activeScreenOf(CATALOG, 'project-detail')?.label).toBe('Projects')
    expect(activeScreenOf(CATALOG, 'console')).toBeNull()
    expect(activeScreenOf(CATALOG, null)).toBeNull()
  })
})
