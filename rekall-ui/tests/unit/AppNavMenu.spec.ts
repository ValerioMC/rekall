import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter, type Router } from 'vue-router'
import { defineComponent } from 'vue'
import AppNavMenu from '@/components/console/AppNavMenu.vue'
import { NAV_GROUPS, activeScreenIn, alertLabel, alertTone, screensOf } from '@/model/nav-tab'
import { useVulnerabilityStore } from '@/features/vulnerability/store/vulnerability.store'
import { useConsoleStore } from '@/stores/console.store'
import { exposure } from './features/vulnerability/fixtures'
import { taskFixture } from './support/taskFixture'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

vi.mock('@/features/vulnerability/api/vulnerabilities.api')

const Blank = defineComponent({ template: '<div />' })

function routerAt(): Router {
  const names = screensOf(NAV_GROUPS).flatMap((screen) => screen.routes.map((name) => ({ name, path: name === 'console' ? '/' : `/${name}` })))
  return createRouter({ history: createMemoryHistory(), routes: names.map((route) => ({ ...route, component: Blank })) })
}

function handedIn(id: string): Task {
  return { ...taskFixture(id as TaskId), reviewActive: true, reviewState: 'CLAIMED' }
}

async function mountMenu(path: string) {
  const router = routerAt()
  await router.push(path)
  await router.isReady()
  const wrapper = mount(AppNavMenu, { global: { plugins: [router] }, attachTo: document.body })
  return { wrapper, router }
}

describe('the screen menu', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    setActivePinia(createPinia())
  })

  it('wears the screen on display and opens every screen under its section', async () => {
    const { wrapper } = await mountMenu('/calendar')
    expect(wrapper.get('[data-testid="nav-menu-trigger"]').text()).toContain('Calendar')

    await wrapper.get('[data-testid="nav-menu-trigger"]').trigger('click')
    await flushPromises()

    const items = wrapper.findAll('[data-testid="nav-menu-item"]')
    expect(items.map((item) => item.find('span').text())).toEqual(screensOf(NAV_GROUPS).map((screen) => screen.label))
    const list = wrapper.get('[data-testid="nav-menu-list"]').text()
    NAV_GROUPS.forEach((group) => expect(list).toContain(group.label))
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

  it('sums the alerts of every screen on its key so folding never hides a warning', async () => {
    useVulnerabilityStore().exposures = [exposure('a', 'p'), exposure('b', 'p', { open: false })]
    const { wrapper } = await mountMenu('/')
    expect(wrapper.get('[data-testid="nav-menu-trigger"] [data-testid="nav-alert-count"]').text()).toBe('1')

    await wrapper.get('[data-testid="nav-menu-trigger"]').trigger('click')
    await flushPromises()
    const security = wrapper.findAll('[data-testid="nav-menu-item"]').find((item) => item.text().includes('Security'))
    expect(security?.get('[data-testid="nav-alert-count"]').attributes('aria-label')).toBe('1 critical advisory reaches your projects')
  })

  it('counts the review queue on Work and on the key', async () => {
    useConsoleStore().tasks = [handedIn('a'), handedIn('b')]
    const { wrapper } = await mountMenu('/projects')
    expect(wrapper.get('[data-testid="nav-menu-trigger"] [data-testid="nav-alert-count"]').text()).toBe('2')

    await wrapper.get('[data-testid="nav-menu-trigger"]').trigger('click')
    await flushPromises()
    const work = wrapper.findAll('[data-testid="nav-menu-item"]').find((item) => item.text().startsWith('Work'))
    expect(work?.get('[data-testid="nav-alert-count"]').attributes('aria-label')).toBe('2 waiting for your review')
  })
})

describe('the menu model', () => {
  it('finds the screen on display', () => {
    expect(activeScreenIn(NAV_GROUPS, 'project-detail')?.label).toBe('Projects')
    expect(activeScreenIn(NAV_GROUPS, 'console')?.label).toBe('Work')
    expect(activeScreenIn(NAV_GROUPS, null)).toBeNull()
  })

  it('words an alert by what it counts', () => {
    expect(alertLabel('open-vulnerabilities', 3)).toBe('3 critical advisories reach your projects')
    expect(alertLabel('review-queue', 1)).toBe('1 waiting for your review')
    expect(alertLabel(undefined, 2)).toBe('2 need attention')
  })

  it('draws a risk in the danger tone and what waits for the reader in amber', () => {
    expect(alertTone('open-vulnerabilities')).toBe('danger')
    expect(alertTone('review-queue')).toBe('accent')
  })
})
