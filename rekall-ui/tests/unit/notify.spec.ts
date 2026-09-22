import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  claimNotificationsEnabled,
  postNotice,
  requestNotificationPermission,
  setClaimNotificationsEnabled
} from '@/common/native/notify'

const notice = { title: 'A step is waiting for review', body: 'Wire the export — Report builder' }

/** A fresh in-memory Storage per test: the preference is per machine, and each test is a new machine. */
function memoryStorage(): Storage {
  const values = new Map<string, string>()
  return {
    get length() {
      return values.size
    },
    clear: () => values.clear(),
    getItem: (key) => values.get(key) ?? null,
    key: (index) => [...values.keys()][index] ?? null,
    removeItem: (key) => void values.delete(key),
    setItem: (key, value) => void values.set(key, String(value))
  }
}

describe('posting a notice', () => {
  beforeEach(() => {
    Object.defineProperty(window, 'localStorage', { value: memoryStorage(), configurable: true })
  })

  afterEach(() => {
    delete window.rekallDesktop
    vi.unstubAllGlobals()
  })

  it('goes to the native host when Rekall.app installed one, and asks the browser nothing', async () => {
    const notify = vi.fn(async () => true)
    window.rekallDesktop = { pickFolder: vi.fn(), notify }
    const Browser = vi.fn()
    vi.stubGlobal('Notification', Browser)

    expect(await postNotice(notice)).toBe(true)
    expect(notify).toHaveBeenCalledWith(notice)
    expect(Browser).not.toHaveBeenCalled()
    expect(await requestNotificationPermission()).toBe(true)
  })

  it('uses the browser only once it has granted permission', async () => {
    const Browser = Object.assign(vi.fn(), { permission: 'default', requestPermission: vi.fn(async () => 'granted') })
    vi.stubGlobal('Notification', Browser)

    expect(await postNotice(notice)).toBe(false)
    expect(Browser).not.toHaveBeenCalled()

    Browser.permission = 'granted'
    expect(await postNotice(notice)).toBe(true)
    expect(Browser).toHaveBeenCalledWith(notice.title, expect.objectContaining({ body: notice.body }))
  })

  it('posts nothing on a machine that turned it off', async () => {
    const notify = vi.fn(async () => true)
    window.rekallDesktop = { pickFolder: vi.fn(), notify }

    setClaimNotificationsEnabled(false)

    expect(claimNotificationsEnabled()).toBe(false)
    expect(await postNotice(notice)).toBe(false)
    expect(notify).not.toHaveBeenCalled()
  })
})
