import { describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { VersionStatus } from '@/model/version'

// A plain function, not a spy: a spy re-throws the rejection it records, which Vitest reports as
// unhandled even though the component caught it.
let answer: () => Promise<VersionStatus> = () => Promise.reject(new Error('not set'))

vi.mock('@/api/version.api', () => ({ fetchVersionStatus: () => answer() }))

import AppVersionLabel from '@/components/shell/AppVersionLabel.vue'

const RELEASE = 'https://github.com/ValerioMC/rekall/releases/tag/v0.2.0'
const DMG = 'https://github.com/ValerioMC/rekall/releases/download/v0.2.0/Rekall-macos-arm64.dmg'

async function mounted() {
  const wrapper = mount(AppVersionLabel)
  await flushPromises()
  return wrapper
}

describe('AppVersionLabel', () => {
  it('shows the running version', async () => {
    answer = () => Promise.resolve({ current: '0.1.0', check: 'UP_TO_DATE', latest: null })

    const wrapper = await mounted()

    expect(wrapper.text()).toContain('v0.1.0')
    expect(wrapper.find('[data-testid="app-update"]').exists()).toBe(false)
  })

  it('offers the download of a newer release', async () => {
    answer = () =>
      Promise.resolve({
        current: '0.1.0',
        check: 'UPDATE_AVAILABLE',
        latest: { version: '0.2.0', releaseUrl: RELEASE, downloadUrl: DMG }
      })

    const link = (await mounted()).get('[data-testid="app-update"]')

    expect(link.text()).toContain('v0.2.0')
    expect(link.attributes('href')).toBe(DMG)
  })

  it('falls back to the release page when the release has no asset for this platform', async () => {
    answer = () =>
      Promise.resolve({
        current: '0.1.0',
        check: 'UPDATE_AVAILABLE',
        latest: { version: '0.2.0', releaseUrl: RELEASE, downloadUrl: null }
      })

    expect((await mounted()).get('[data-testid="app-update"]').attributes('href')).toBe(RELEASE)
  })

  it('shows nothing when the version cannot be read', async () => {
    answer = () => Promise.reject(new Error('offline'))

    expect((await mounted()).find('[data-testid="app-version"]').exists()).toBe(false)
  })
})
