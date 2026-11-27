import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent } from 'vue'
import { createMemoryHistory, createRouter } from 'vue-router'
import { setPreferredProvider } from '@/common/config/session-launch'
import { useCodexStore } from '@/features/codex/store/codex.store'
import SharedSessionSection from '@/features/settings/components/SharedSessionSection.vue'
import { installStorage } from './support/memoryStorage'
import AppHeaderChrome from '@/components/shell/AppHeaderChrome.vue'

vi.mock('@/features/vulnerability/api/vulnerabilities.api')

describe('the header drag region', () => {
  afterEach(() => {
    delete window.rekallDesktop
  })

  it('is deep in the desktop window, so a press on any empty part of the bar drags it', () => {
    window.rekallDesktop = { pickFolder: async () => null, platform: 'macos' }
    const wrapper = shallowMount(AppHeaderChrome)
    expect(wrapper.get('header').attributes('data-tauri-drag-region')).toBe('deep')
  })

  it('is absent in a browser tab', () => {
    const wrapper = shallowMount(AppHeaderChrome)
    expect(wrapper.get('header').attributes('data-tauri-drag-region')).toBeUndefined()
  })
})

describe('the header session provider', () => {
  beforeEach(() => {
    installStorage()
    setActivePinia(createPinia())
  })

  it('sits between the search and the actions, ahead of the live status', async () => {
    const { header } = await mountHeader()
    const order = ['#app-header-extra', '[data-testid="header-session-provider"]', '#app-header-status', '#app-header-actions']

    const found = [...header.element.querySelectorAll(order.join(', '))]
    expect(found.map((element) => order.find((selector) => element.matches(selector)))).toEqual(order)
    expect(header.get('[data-testid="header-session"]').find('#app-header-status').exists()).toBe(true)
    header.unmount()
  })

  it('stands in its own keycap outside the status capsule, tinted for the provider it holds', async () => {
    const { header } = await mountHeader()
    const key = header.get('[data-testid="header-session-provider"]')
    expect(header.get('#app-header-status').find('[data-testid="header-session-provider"]').exists()).toBe(false)
    expect(key.classes()).toContain('provider-keycap')
    expect(key.classes()).toContain('[--provider-tint:var(--color-brand-claude)]')

    await key.trigger('click')
    providerOption('Codex').click()
    await flushPromises()
    expect(key.classes()).toContain('[--provider-tint:var(--color-text)]')
    header.unmount()
    setPreferredProvider('CLAUDE')
  })

  it('selects by keyboard and preserves the choice across navigation and remount', async () => {
    const { header, router } = await mountHeader()
    const key = header.get('[data-testid="header-session-provider"]')
    expect(key.attributes('aria-label')).toContain('Claude Code')
    expect(key.text()).toBe('')
    expect(key.findAll('svg')).toHaveLength(1)
    expect(key.get('svg').attributes('data-provider')).toBe('CLAUDE')

    await key.trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).toBe(providerOption('Claude Code'))
    document.activeElement?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))
    expect(document.activeElement).toBe(providerOption('Codex'))
    providerOption('Codex').click()
    await flushPromises()
    expect(document.activeElement).toBe(key.element)

    await router.push('/tasks/2')
    expect(key.attributes('aria-label')).toContain('Codex')
    expect(key.get('svg').attributes('data-provider')).toBe('CODEX')
    expect(window.localStorage.getItem('rekall.session.provider')).toBe('CODEX')
    header.unmount()

    const restored = await mountHeader()
    expect(restored.header.get('[data-testid="header-session-provider"]').attributes('aria-label')).toContain('Codex')
    restored.header.unmount()
  })

  it('shares the provider with Settings when storage refuses writes', async () => {
    installStorage({ setItem: () => { throw new Error('storage is unavailable') } })
    const { header } = await mountHeader()
    const settings = mount(SharedSessionSection)
    const key = header.get('[data-testid="header-session-provider"]')

    await key.trigger('click')
    providerOption('Codex').click()
    await flushPromises()
    expect(settings.get('[data-testid="segment-CODEX"]').attributes('aria-pressed')).toBe('true')

    await settings.get('[data-testid="segment-CLAUDE"]').trigger('click')
    expect(key.attributes('aria-label')).toContain('Claude Code')
    header.unmount()
    settings.unmount()
    installStorage()
    setPreferredProvider('CLAUDE')
  })

  it('shows an unavailable selection and links to agent setup', async () => {
    setPreferredProvider('CODEX')
    useCodexStore().setup = { status: 'CLI_MISSING' }
    const { header } = await mountHeader()
    const key = header.get('[data-testid="header-session-provider"]')

    expect(key.attributes('aria-label')).toContain('Codex unavailable')
    expect(key.classes()).toContain('[--provider-edge:var(--color-warn)]')
    await key.trigger('click')
    expect(providerOption('Codex').getAttribute('aria-disabled')).toBe('true')
    expect(document.querySelector('[role="menu"] a')?.getAttribute('href')).toBe('/settings/sessions')
    header.unmount()
  })
})

async function mountHeader() {
  const Blank = defineComponent({ template: '<div />' })
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [{ path: '/tasks/:id', component: Blank }, { path: '/settings/sessions', component: Blank }]
  })
  await router.push('/tasks/1')
  const header = mount(AppHeaderChrome, {
    attachTo: document.body,
    global: { plugins: [router] }
  })
  return { header, router }
}

function providerOption(label: string): HTMLButtonElement {
  return [...document.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]')]
    .find((option) => option.textContent?.includes(label))!
}
