import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import { defineComponent } from 'vue'
import AppNavMenu from '@/components/console/AppNavMenu.vue'
import { NAV_ENTRIES, activeScreenIn, alertLabel, screensOf } from '@/model/nav-tab'
import { useVulnerabilityStore } from '@/features/vulnerability/store/vulnerability.store'
import { exposure } from './features/vulnerability/fixtures'

vi.mock('@/features/vulnerability/api/vulnerabilities.api')

const Blank = defineComponent({ template: '<div />' })

function routerAt(): Router {
  const names = NAV_ENTRIES.flatMap(screensOf).flatMap((screen) => screen.routes.map((name) => ({ name, path: name === 'console' ? '/' : `/${name}` })))
  return createRouter({ history: createMemoryHistory(), routes: names.map((route) => ({ ...route, component: Blank })) })
}

async function mountMenu(path: string) {
  const router = routerAt()
  await router.push(path)
  await router.isReady()
  const wrapper = mount(AppNavMenu, { global: { plugins: [router] }, attachTo: document.body })
  return { wrapper, router }
}

describe('the folded screen switcher', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    setActivePinia(createPinia())
  })

  it('wears the screen on display and opens every screen as one vertical list', async () => {
    const { wrapper } = await mountMenu('/calendar')
    expect(wrapper.get('[data-testid="nav-menu-trigger"]').text()).toContain('Calendar')

    await wrapper.get('[data-testid="nav-menu-trigger"]').trigger('click')
    await flushPromises()

    const items = wrapper.findAll('[data-testid="nav-menu-item"]')
    expect(items.map((item) => item.find('span > span').text())).toEqual(NAV_ENTRIES.flatMap(screensOf).map((screen) => screen.label))
    expect(wrapper.get('[data-testid="nav-menu-list"]').text()).toContain('Catalog')
    expect(items.find((item) => item.attributes('aria-current') === 'page')?.text()).toContain('Calendar')
    expect(document.activeElement?.textContent).toContain('Calendar')
  })

  it('closes on Escape and gives the focus back to its key', async () => {
    const { wrapper } = await mountMenu('/')
    await wrapper.get('[data-testid="nav-menu-trigger"]').trigger('click')
    await flushPromises()
    await wrapper.get('[data-testid="nav-menu-list"]').trigger('keydown', { key: 'Escape' })
    expect(wrapper.find('[data-testid="nav-menu-list"]').exists()).toBe(false)
    expect(document.activeElement).toBe(wrapper.get('[data-testid="nav-menu-trigger"]').element)
  })

  it('carries an alert on its key so folding the tabs never hides a warning', async () => {
    useVulnerabilityStore().exposures = [exposure('a', 'p'), exposure('b', 'p', { open: false })]
    const { wrapper } = await mountMenu('/')
    expect(wrapper.get('[data-testid="nav-menu-trigger"] [data-testid="nav-alert-count"]').text()).toBe('1')

    await wrapper.get('[data-testid="nav-menu-trigger"]').trigger('click')
    await flushPromises()
    const security = wrapper.findAll('[data-testid="nav-menu-item"]').find((item) => item.text().includes('Security'))
    expect(security?.get('[data-testid="nav-alert-count"]').attributes('aria-label')).toBe('1 critical advisory reaches your projects')
  })
})

describe('the switcher model', () => {
  it('finds the screen on display in a group or on its own', () => {
    expect(activeScreenIn(NAV_ENTRIES, 'project-detail')?.label).toBe('Projects')
    expect(activeScreenIn(NAV_ENTRIES, 'security')?.label).toBe('Security')
    expect(activeScreenIn(NAV_ENTRIES, null)).toBeNull()
  })

  it('words an alert by what it counts', () => {
    expect(alertLabel('open-vulnerabilities', 3)).toBe('3 critical advisories reach your projects')
    expect(alertLabel(undefined, 2)).toBe('2 need attention')
  })
})
