import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { defineComponent } from 'vue'
import HeaderNavigation from '@/components/shell/HeaderNavigation.vue'
import { NAV_GROUPS, screensOf } from '@/model/nav-tab'

vi.mock('@/features/vulnerability/api/vulnerabilities.api')

const Blank = defineComponent({ template: '<div />' })

async function mountNavigation() {
  const routes = screensOf(NAV_GROUPS).flatMap((screen) =>
    screen.routes.map((name) => ({ name, path: name === 'console' ? '/' : `/${name}`, component: Blank }))
  )
  const router = createRouter({ history: createMemoryHistory(), routes })
  await router.push('/')
  await router.isReady()
  const wrapper = mount(HeaderNavigation, { global: { plugins: [router] }, attachTo: document.body })
  return { router, wrapper }
}

describe('the header navigation', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    setActivePinia(createPinia())
  })

  it('orders history, sections and project in the DOM', async () => {
    const { wrapper } = await mountNavigation()
    const order = ['history-back', 'history-forward', 'nav-menu-trigger', 'header-project-switch']
    const selector = order.map((id) => `[data-testid="${id}"]`).join(', ')

    expect(wrapper.findAll(selector).map((control) => control.attributes('data-testid'))).toEqual(order)
    expect(wrapper.find('[data-testid="home-link"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="header-session-provider"]').exists()).toBe(false)
  })

  it('opens the Projects list from the section menu', async () => {
    const { router, wrapper } = await mountNavigation()
    await wrapper.get('[data-testid="nav-menu-trigger"]').trigger('click')
    await wrapper.get('[data-testid="nav-menu-list"] a[href="/projects"]').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('projects')
  })

  it('folds the menu back on a press outside it', async () => {
    const { wrapper } = await mountNavigation()
    await wrapper.get('[data-testid="nav-menu-trigger"]').trigger('click')
    document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
    await flushPromises()
    expect(wrapper.find('[data-testid="nav-menu-list"]').exists()).toBe(false)
  })
})
