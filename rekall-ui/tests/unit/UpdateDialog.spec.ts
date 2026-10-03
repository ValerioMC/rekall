import { afterEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import UpdateDialog from '@/components/shell/UpdateDialog.vue'

const DMG = 'https://github.com/ValerioMC/rekall/releases/download/v0.2.0/Rekall-macos-arm64.dmg'

function installBridge(installUpdate: () => Promise<boolean>): void {
  window.rekallDesktop = { pickFolder: vi.fn(), installUpdate }
}

function mountDialog() {
  return mount(UpdateDialog, { props: { version: '0.2.0', downloadHref: DMG } })
}

afterEach(() => {
  delete window.rekallDesktop
})

describe('UpdateDialog', () => {
  it('installs through the desktop bridge and stays open while the app restarts', async () => {
    const installUpdate = vi.fn(() => Promise.resolve(true))
    installBridge(installUpdate)
    const wrapper = mountDialog()

    await wrapper.get('[data-testid="update-install"]').trigger('click')
    await flushPromises()

    expect(installUpdate).toHaveBeenCalledOnce()
    expect(wrapper.get('[data-testid="update-install"]').text()).toBe('Installing…')
    expect(wrapper.emitted('close')).toBeUndefined()
  })

  it('closes when the user keeps a live Claude session instead', async () => {
    installBridge(() => Promise.resolve(false))
    const wrapper = mountDialog()

    await wrapper.get('[data-testid="update-install"]').trigger('click')
    await flushPromises()

    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('after a failed install says why and offers the download in the browser', async () => {
    installBridge(() => Promise.reject(new Error('the disk image could not be opened')))
    const wrapper = mountDialog()

    await wrapper.get('[data-testid="update-install"]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="update-failure"]').text()).toContain('the disk image could not be opened')
    expect(wrapper.get('[data-testid="update-download"]').attributes('href')).toBe(DMG)
    expect(wrapper.find('[data-testid="update-install"]').exists()).toBe(false)
  })

  it('cannot be dismissed while the install runs', async () => {
    installBridge(() => new Promise<boolean>(() => {}))
    const wrapper = mountDialog()

    await wrapper.get('[data-testid="update-install"]').trigger('click')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))

    expect(wrapper.emitted('close')).toBeUndefined()
  })
})
