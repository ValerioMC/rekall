import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useUpdateStore } from '@/stores/update.store'
import { fetchVersionStatus } from '@/api/version.api'
import type { VersionStatus } from '@/model/version'

vi.mock('@/api/version.api', () => ({ fetchVersionStatus: vi.fn() }))

const available: VersionStatus = {
  current: '0.1.0',
  check: 'UPDATE_AVAILABLE',
  latest: { version: '0.2.0', releaseUrl: 'https://example.test/release', downloadUrl: null }
}
const upToDate: VersionStatus = { current: '0.2.0', check: 'UP_TO_DATE', latest: null }

beforeEach(() => {
  setActivePinia(createPinia())
  vi.mocked(fetchVersionStatus).mockReset()
})

afterEach(() => {
  delete window.rekallDesktop
})

describe('update store', () => {
  it('opens the prompt at startup in the desktop app when a newer release exists', async () => {
    window.rekallDesktop = { pickFolder: vi.fn() }
    vi.mocked(fetchVersionStatus).mockResolvedValue(available)
    const update = useUpdateStore()

    await update.checkAtStartup()

    expect(update.promptOpen).toBe(true)
    expect(update.downloadHref).toBe('https://example.test/release')
  })

  it('stays silent at startup when up to date', async () => {
    window.rekallDesktop = { pickFolder: vi.fn() }
    vi.mocked(fetchVersionStatus).mockResolvedValue(upToDate)
    const update = useUpdateStore()

    await update.checkAtStartup()

    expect(update.promptOpen).toBe(false)
  })

  it('stays silent at startup when GitHub cannot be reached', async () => {
    window.rekallDesktop = { pickFolder: vi.fn() }
    vi.mocked(fetchVersionStatus).mockRejectedValue(new Error('offline'))
    const update = useUpdateStore()

    await update.checkAtStartup()

    expect(update.promptOpen).toBe(false)
  })

  it('never prompts in a browser tab and checks once per session', async () => {
    vi.mocked(fetchVersionStatus).mockResolvedValue(available)
    const update = useUpdateStore()

    await update.checkAtStartup()
    expect(fetchVersionStatus).not.toHaveBeenCalled()

    window.rekallDesktop = { pickFolder: vi.fn() }
    await update.checkAtStartup()
    await update.checkAtStartup()
    expect(fetchVersionStatus).toHaveBeenCalledOnce()
  })

  it('asks the server to refresh when the check is by hand', async () => {
    vi.mocked(fetchVersionStatus).mockResolvedValue(upToDate)
    const update = useUpdateStore()

    await update.check(true)

    expect(fetchVersionStatus).toHaveBeenCalledWith(true)
  })
})
