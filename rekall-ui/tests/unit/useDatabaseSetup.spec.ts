import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useDatabaseSetup } from '@/composables/useDatabaseSetup'

/**
 * The shared machinery behind the folder field, the unreachable-database screen and Settings:
 * debouncing the live check, and the submit-then-wait-for-restart-then-reload sequence every
 * mutating action goes through. Mounting a component to exercise this would test Vue; this
 * tests the sequencing, which is the part that is easy to get subtly wrong.
 */
const addDatabase = vi.fn()
const activateDatabase = vi.fn()
const checkFolder = vi.fn()

vi.mock('@/api/settings.api', () => ({
  addDatabase: (...args: unknown[]) => addDatabase(...args),
  activateDatabase: (...args: unknown[]) => activateDatabase(...args),
  checkFolder: (...args: unknown[]) => checkFolder(...args)
}))

const anEntry = {
  id: '1',
  label: 'Local',
  path: '/a',
  active: true,
  reachable: true,
  addedAt: 't',
  lastUsedAt: 't'
}

describe('useDatabaseSetup', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    addDatabase.mockReset()
    activateDatabase.mockReset()
    checkFolder.mockReset()
    vi.stubGlobal('fetch', vi.fn())
    vi.stubGlobal('location', { ...window.location, reload: vi.fn() })
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.unstubAllGlobals()
  })

  it('debounces the live check to the last path typed', async () => {
    checkFolder.mockResolvedValue({
      resolvedPath: '/abc',
      exists: true,
      isDirectory: true,
      writable: true,
      hasDatabase: false,
      usable: true
    })
    const { checkPath, check } = useDatabaseSetup()

    checkPath('/a')
    checkPath('/ab')
    checkPath('/abc')
    await vi.advanceTimersByTimeAsync(350)

    expect(checkFolder).toHaveBeenCalledTimes(1)
    expect(checkFolder).toHaveBeenCalledWith('/abc')
    expect(check.value?.usable).toBe(true)
  })

  it('clears the check result rather than checking a blank path', async () => {
    checkFolder.mockResolvedValue({
      resolvedPath: '/a',
      exists: true,
      isDirectory: true,
      writable: true,
      hasDatabase: false,
      usable: true
    })
    const { checkPath, check } = useDatabaseSetup()

    checkPath('/a')
    await vi.advanceTimersByTimeAsync(350)
    expect(check.value).not.toBeNull()

    checkPath('   ')

    expect(check.value).toBeNull()
    expect(checkFolder).toHaveBeenCalledTimes(1)
  })

  it('reloads the page once the server answers healthy again after adding a folder', async () => {
    addDatabase.mockResolvedValue({ mode: 'created', entry: anEntry })
    vi.mocked(fetch).mockResolvedValue({ ok: true } as Response)
    const { submitNewFolder, phase } = useDatabaseSetup()

    const pending = submitNewFolder('/a')
    await vi.runAllTimersAsync()
    await pending

    expect(window.location.reload).toHaveBeenCalledOnce()
    expect(phase.value).toBe('restarting')
  })

  it('reloads the page once the server answers healthy again after switching', async () => {
    activateDatabase.mockResolvedValue(anEntry)
    vi.mocked(fetch).mockResolvedValue({ ok: true } as Response)
    const { switchTo, phase } = useDatabaseSetup()

    const pending = switchTo(anEntry)
    await vi.runAllTimersAsync()
    await pending

    expect(activateDatabase).toHaveBeenCalledWith('1')
    expect(window.location.reload).toHaveBeenCalledOnce()
    expect(phase.value).toBe('restarting')
  })

  it('surfaces the server error and never restarts when the add is rejected', async () => {
    addDatabase.mockRejectedValue(new Error('The folder "/a" does not exist. Create it, then try again.'))
    const { submitNewFolder, phase, error, timedOut } = useDatabaseSetup()

    await submitNewFolder('/a')

    expect(phase.value).toBe('error')
    expect(error.value).toContain('does not exist')
    expect(timedOut.value).toBe(false)
    expect(fetch).not.toHaveBeenCalled()
    expect(window.location.reload).not.toHaveBeenCalled()
  })

  it('gives up and reports an error if the server never comes back', async () => {
    addDatabase.mockResolvedValue({ mode: 'created', entry: anEntry })
    vi.mocked(fetch).mockRejectedValue(new Error('connection refused'))
    const { submitNewFolder, phase, error, timedOut } = useDatabaseSetup()

    const pending = submitNewFolder('/a')
    await vi.runAllTimersAsync()
    await pending

    expect(phase.value).toBe('error')
    expect(error.value).toContain('longer than expected')
    expect(timedOut.value).toBe(true)
    expect(window.location.reload).not.toHaveBeenCalled()
  })
})
