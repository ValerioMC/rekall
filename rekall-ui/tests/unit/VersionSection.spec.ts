import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import VersionSection from '@/components/settings/VersionSection.vue'
import { fetchVersionStatus } from '@/api/version.api'
import type { VersionStatus } from '@/model/version'

vi.mock('@/api/version.api', () => ({ fetchVersionStatus: vi.fn() }))

const DMG = 'https://github.com/ValerioMC/rekall/releases/download/v0.2.0/Rekall-macos-arm64.dmg'
const available: VersionStatus = {
  current: '0.1.0',
  check: 'UPDATE_AVAILABLE',
  latest: { version: '0.2.0', releaseUrl: 'https://example.test/release', downloadUrl: DMG }
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.mocked(fetchVersionStatus).mockReset()
})

afterEach(() => {
  delete window.rekallDesktop
})

describe('VersionSection', () => {
  it('shows the running version and that it is up to date', async () => {
    vi.mocked(fetchVersionStatus).mockResolvedValue({ current: '0.2.0', check: 'UP_TO_DATE', latest: null })
    const wrapper = mount(VersionSection)
    await flushPromises()

    expect(wrapper.get('[data-testid="version-current"]').text()).toBe('Rekall v0.2.0')
    expect(wrapper.text()).toContain('Up to date')
  })

  it('installs a newer release from the app on macOS', async () => {
    const installUpdate = vi.fn(() => Promise.resolve(true))
    window.rekallDesktop = { pickFolder: vi.fn(), installUpdate, installsUpdates: true }
    vi.mocked(fetchVersionStatus).mockResolvedValue(available)
    const wrapper = mount(VersionSection)
    await flushPromises()

    await wrapper.get('[data-testid="version-install"]').trigger('click')

    expect(installUpdate).toHaveBeenCalledOnce()
  })

  it('links the download where the window cannot install', async () => {
    vi.mocked(fetchVersionStatus).mockResolvedValue(available)
    const wrapper = mount(VersionSection)
    await flushPromises()

    expect(wrapper.find('[data-testid="version-install"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="version-download"]').attributes('href')).toBe(DMG)
  })

  it('checks again on request, bypassing what the server remembered', async () => {
    vi.mocked(fetchVersionStatus).mockResolvedValue({ current: '0.2.0', check: 'UP_TO_DATE', latest: null })
    const wrapper = mount(VersionSection)
    await flushPromises()

    await wrapper.get('[data-testid="version-check"]').trigger('click')
    await flushPromises()

    expect(fetchVersionStatus).toHaveBeenLastCalledWith(true)
  })

  it('says when the check failed', async () => {
    vi.mocked(fetchVersionStatus).mockRejectedValue(new Error('Rekall could not be reached'))
    const wrapper = mount(VersionSection)
    await flushPromises()

    expect(wrapper.get('[role="alert"]').text()).toContain('Rekall could not be reached')
  })
})
