import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import UpdateDialog from '@/components/shell/UpdateDialog.vue'
import { useUpdateStore } from '@/stores/update.store'
import type { VersionStatus } from '@/model/version'

const DMG = 'https://github.com/ValerioMC/rekall/releases/download/v0.2.0/Rekall-macos-arm64.dmg'

vi.mock('@/api/version.api', () => ({ fetchVersionStatus: vi.fn() }))

const available: VersionStatus = {
  current: '0.1.0',
  check: 'UPDATE_AVAILABLE',
  latest: { version: '0.2.0', releaseUrl: 'https://github.com/ValerioMC/rekall/releases/tag/v0.2.0', downloadUrl: DMG }
}

function installBridge(installUpdate: () => Promise<boolean>, installsUpdates = true): void {
  window.rekallDesktop = { pickFolder: vi.fn(), installUpdate, installsUpdates }
}

function mountDialog() {
  const update = useUpdateStore()
  update.status = available
  update.promptOpen = true
  return { update, wrapper: mount(UpdateDialog) }
}

beforeEach(() => setActivePinia(createPinia()))

afterEach(() => {
  delete window.rekallDesktop
})

describe('UpdateDialog', () => {
  it('installs through the desktop bridge and stays open while the app restarts', async () => {
    const installUpdate = vi.fn(() => Promise.resolve(true))
    installBridge(installUpdate)
    const { update, wrapper } = mountDialog()

    await wrapper.get('[data-testid="update-install"]').trigger('click')
    await flushPromises()

    expect(installUpdate).toHaveBeenCalledOnce()
    expect(wrapper.get('[data-testid="update-install"]').text()).toBe('Installing…')
    expect(update.promptOpen).toBe(true)
  })

  it('closes when the user keeps a live Claude session instead', async () => {
    installBridge(() => Promise.resolve(false))
    const { update, wrapper } = mountDialog()

    await wrapper.get('[data-testid="update-install"]').trigger('click')
    await flushPromises()

    expect(update.promptOpen).toBe(false)
  })

  it('after a failed install says why and offers the download in the browser', async () => {
    installBridge(() => Promise.reject(new Error('the disk image could not be opened')))
    const { wrapper } = mountDialog()

    await wrapper.get('[data-testid="update-install"]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="update-failure"]').text()).toContain('the disk image could not be opened')
    expect(wrapper.get('[data-testid="update-download"]').attributes('href')).toBe(DMG)
    expect(wrapper.find('[data-testid="update-install"]').exists()).toBe(false)
  })

  it('offers only the download where the window cannot install', () => {
    installBridge(() => Promise.resolve(true), false)
    const { wrapper } = mountDialog()

    expect(wrapper.find('[data-testid="update-install"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="update-download"]').attributes('href')).toBe(DMG)
  })

  it('cannot be dismissed while the install runs', async () => {
    installBridge(() => new Promise<boolean>(() => {}))
    const { update, wrapper } = mountDialog()

    await wrapper.get('[data-testid="update-install"]').trigger('click')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))

    expect(update.promptOpen).toBe(true)
  })

  it('Later closes it', async () => {
    installBridge(() => Promise.resolve(true))
    const { update, wrapper } = mountDialog()

    await wrapper.findAll('button').find((button) => button.text() === 'Later')!.trigger('click')

    expect(update.promptOpen).toBe(false)
  })
})
