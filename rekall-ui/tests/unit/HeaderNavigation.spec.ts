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
  return mount(HeaderNavigation, { global: { plugins: [router] }, attachTo: document.body })
}

describe('the header navigation', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    setActivePinia(createPinia())
  })

  it('keeps the menu key and the history arrows side by side', async () => {
    const wrapper = await mountNavigation()
    expect(wrapper.find('[data-testid="nav-menu-trigger"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="history-back"]').exists()).toBe(true)
  })

  it('opens the menu without taking the history arrows away', async () => {
    const wrapper = await mountNavigation()
    await wrapper.get('[data-testid="nav-menu-trigger"]').trigger('click')
    await flushPromises()
    expect(wrapper.find('[data-testid="nav-menu-list"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="history-back"]').exists()).toBe(true)
  })

  it('folds the menu back on a press outside it', async () => {
    const wrapper = await mountNavigation()
    await wrapper.get('[data-testid="nav-menu-trigger"]').trigger('click')
    document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
    await flushPromises()
    expect(wrapper.find('[data-testid="nav-menu-list"]').exists()).toBe(false)
  })
})
