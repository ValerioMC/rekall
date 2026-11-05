import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { defineComponent } from 'vue'
import HeaderNavigation from '@/components/shell/HeaderNavigation.vue'
import { NAV_ENTRIES, screensOf } from '@/model/nav-tab'

vi.mock('@/features/vulnerability/api/vulnerabilities.api')

const Blank = defineComponent({ template: '<div />' })

async function mountNavigation() {
  const routes = NAV_ENTRIES.flatMap(screensOf).flatMap((screen) =>
    screen.routes.map((name) => ({ name, path: name === 'console' ? '/' : `/${name}`, component: Blank }))
  )
  const router = createRouter({ history: createMemoryHistory(), routes })
  await router.push('/')
  await router.isReady()
  return mount(HeaderNavigation, { global: { plugins: [router] }, attachTo: document.body })
}

const toggle = (wrapper: Awaited<ReturnType<typeof mountNavigation>>) => wrapper.get('[data-testid="header-menu-toggle"]')

describe('the header menu key', () => {
  beforeEach(() => {
    document.body.innerHTML = ''
    setActivePinia(createPinia())
  })

  it('starts folded: the key and the history arrows, no screen tabs', async () => {
    const wrapper = await mountNavigation()
    expect(toggle(wrapper).attributes('aria-expanded')).toBe('false')
    expect(wrapper.find('[data-testid="history-back"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="nav-tabs"]').exists()).toBe(false)
  })

  it('opens the screen tabs in place of the history arrows', async () => {
    const wrapper = await mountNavigation()
    await toggle(wrapper).trigger('click')
    expect(toggle(wrapper).attributes('aria-expanded')).toBe('true')
    expect(wrapper.find('[data-testid="nav-tabs"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="history-back"]').exists()).toBe(false)
  })

  it('folds back when pressed again', async () => {
    const wrapper = await mountNavigation()
    await toggle(wrapper).trigger('click')
    await toggle(wrapper).trigger('click')
    expect(wrapper.find('[data-testid="history-back"]').exists()).toBe(true)
  })

  it('folds back, arrows returning, on a press outside it', async () => {
    const wrapper = await mountNavigation()
    await toggle(wrapper).trigger('click')
    document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
    await flushPromises()
    expect(wrapper.find('[data-testid="nav-tabs"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="history-back"]').exists()).toBe(true)
  })

  it('stays open on a press inside the tabs', async () => {
    const wrapper = await mountNavigation()
    await toggle(wrapper).trigger('click')
    wrapper.get('[data-testid="nav-tabs"]').element.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
    await flushPromises()
    expect(wrapper.find('[data-testid="nav-tabs"]').exists()).toBe(true)
  })
})
