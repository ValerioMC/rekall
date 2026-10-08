import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createRouter, createWebHistory, type Router } from 'vue-router'
import HistoryArrows from '@/components/shell/HistoryArrows.vue'
import { installFakeBrowserHistory } from './support/fakeBrowserHistory'

/** A web history, as the app runs on: only it records the entries on either side of the current one. */
let router: Router
let wrapper: VueWrapper | null = null
let restoreHistory: () => void = () => undefined

async function press(page: VueWrapper, side: 'back' | 'forward'): Promise<void> {
  await page.get(`[data-testid="history-${side}"]`).trigger('click')
  await flushPromises()
}

async function render(): Promise<VueWrapper> {
  wrapper = mount(HistoryArrows, { global: { plugins: [router] } })
  await flushPromises()
  return wrapper
}

function disabled(page: VueWrapper, side: 'back' | 'forward'): boolean {
  return (page.get(`[data-testid="history-${side}"]`).element as HTMLButtonElement).disabled
}

beforeEach(async () => {
  restoreHistory = installFakeBrowserHistory()
  router = createRouter({
    history: createWebHistory(),
    routes: [{ path: '/:screen(.*)*', component: { template: '<div />' } }]
  })
  await router.push('/')
  await router.isReady()
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  restoreHistory()
})

describe('the history arrows', () => {
  it('offer neither side on the first screen of a visit', async () => {
    const page = await render()

    expect(disabled(page, 'back')).toBe(true)
    expect(disabled(page, 'forward')).toBe(true)
  })

  it('offer back once the reader has moved on', async () => {
    const page = await render()

    await router.push('/projects')
    await flushPromises()

    expect(disabled(page, 'back')).toBe(false)
    expect(disabled(page, 'forward')).toBe(true)
  })

  it('return to the previous screen and then offer the way forward', async () => {
    const page = await render()
    await router.push('/projects')
    await flushPromises()

    await press(page, 'back')

    expect(router.currentRoute.value.fullPath).toBe('/')
    expect(disabled(page, 'back')).toBe(true)
    expect(disabled(page, 'forward')).toBe(false)
  })

  it('go forward again to the screen the reader came back from', async () => {
    const page = await render()
    await router.push('/projects')
    await flushPromises()
    await press(page, 'back')

    await press(page, 'forward')

    expect(router.currentRoute.value.fullPath).toBe('/projects')
    expect(disabled(page, 'forward')).toBe(true)
  })

  it('drop the way forward when the reader takes a new path from an earlier screen', async () => {
    const page = await render()
    await router.push('/projects')
    await flushPromises()
    await press(page, 'back')

    await router.push('/calendar')
    await flushPromises()

    expect(disabled(page, 'back')).toBe(false)
    expect(disabled(page, 'forward')).toBe(true)
  })

  it('answer the Alt+Arrow chord from anywhere on the page', async () => {
    const page = await render()
    await router.push('/projects')
    await flushPromises()

    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'ArrowLeft', altKey: true, metaKey: true }))
    window.dispatchEvent(new KeyboardEvent('keydown', { code: 'ArrowLeft', altKey: true }))
    await flushPromises()

    expect(router.currentRoute.value.fullPath).toBe('/')
    expect(disabled(page, 'back')).toBe(true)
  })
})
