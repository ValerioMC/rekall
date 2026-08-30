import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import DatabaseFolderField from '@/components/setup/DatabaseFolderField.vue'

/**
 * The one control every database-location flow is built on. What matters here is that the
 * three outcomes a typed path can resolve to — missing, already a database, empty and ready to
 * become one — are visibly distinct and only two of them let you submit.
 */
const addDatabase = vi.fn()
const checkFolder = vi.fn()

vi.mock('@/api/settings.api', () => ({
  addDatabase: (...args: unknown[]) => addDatabase(...args),
  activateDatabase: vi.fn(),
  checkFolder: (...args: unknown[]) => checkFolder(...args)
}))

async function typePath(wrapper: ReturnType<typeof mount>, value: string): Promise<void> {
  await wrapper.get('[data-testid="database-folder-input"]').setValue(value)
  await vi.advanceTimersByTimeAsync(350)
}

describe('DatabaseFolderField', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    addDatabase.mockReset()
    checkFolder.mockReset()
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.unstubAllGlobals()
  })

  it('disables submit until a folder has been checked', () => {
    const wrapper = mount(DatabaseFolderField)

    expect(wrapper.get<HTMLButtonElement>('[data-testid="folder-submit"]').element.disabled).toBe(true)
  })

  it('reports a missing folder and keeps submit disabled', async () => {
    checkFolder.mockResolvedValue({
      resolvedPath: '/nowhere',
      exists: false,
      isDirectory: false,
      writable: false,
      hasDatabase: false,
      usable: false
    })
    const wrapper = mount(DatabaseFolderField)

    await typePath(wrapper, '/nowhere')

    expect(wrapper.get('[data-testid="folder-hint"]').text()).toContain("doesn't exist")
    expect(wrapper.get<HTMLButtonElement>('[data-testid="folder-submit"]').element.disabled).toBe(true)
  })

  it('offers to open, not create, when a database already lives at the path', async () => {
    checkFolder.mockResolvedValue({
      resolvedPath: '/a',
      exists: true,
      isDirectory: true,
      writable: true,
      hasDatabase: true,
      usable: true
    })
    const wrapper = mount(DatabaseFolderField)

    await typePath(wrapper, '/a')

    expect(wrapper.get('[data-testid="folder-hint"]').text()).toContain('already lives here')
    expect(wrapper.get('[data-testid="folder-submit"]').text()).toBe('Open this database')
    expect(wrapper.get<HTMLButtonElement>('[data-testid="folder-submit"]').element.disabled).toBe(false)
  })

  it('offers to create a database in an existing, empty folder', async () => {
    checkFolder.mockResolvedValue({
      resolvedPath: '/a',
      exists: true,
      isDirectory: true,
      writable: true,
      hasDatabase: false,
      usable: true
    })
    const wrapper = mount(DatabaseFolderField)

    await typePath(wrapper, '/a')

    expect(wrapper.get('[data-testid="folder-hint"]').text()).toContain('will be created')
    expect(wrapper.get('[data-testid="folder-submit"]').text()).toBe('Create database here')
  })

  it('reports a folder that exists but cannot be written to', async () => {
    checkFolder.mockResolvedValue({
      resolvedPath: '/a',
      exists: true,
      isDirectory: true,
      writable: false,
      hasDatabase: false,
      usable: false
    })
    const wrapper = mount(DatabaseFolderField)

    await typePath(wrapper, '/a')

    expect(wrapper.get('[data-testid="folder-hint"]').text()).toContain("can't write")
    expect(wrapper.get<HTMLButtonElement>('[data-testid="folder-submit"]').element.disabled).toBe(true)
  })

  it('shows what the typed path actually resolved to', async () => {
    checkFolder.mockResolvedValue({
      resolvedPath: '/Users/you/rekall',
      exists: true,
      isDirectory: true,
      writable: true,
      hasDatabase: false,
      usable: true
    })
    const wrapper = mount(DatabaseFolderField)

    await typePath(wrapper, '~/rekall')

    expect(wrapper.get('[data-testid="resolved-path"]').text()).toContain('/Users/you/rekall')
  })

  it('emits cancel rather than submitting when cancellable', async () => {
    const wrapper = mount(DatabaseFolderField, { props: { cancellable: true } })

    await wrapper.get('button[type="button"]').trigger('click')

    expect(wrapper.emitted('cancel')).toHaveLength(1)
    expect(addDatabase).not.toHaveBeenCalled()
  })

  it('disables cancel once a submit is in flight, so it cannot orphan the request it was meant to abandon', async () => {
    checkFolder.mockResolvedValue({
      resolvedPath: '/a',
      exists: true,
      isDirectory: true,
      writable: true,
      hasDatabase: false,
      usable: true
    })
    addDatabase.mockImplementation(() => new Promise(() => {}))
    const wrapper = mount(DatabaseFolderField, { props: { cancellable: true } })

    await typePath(wrapper, '/a')
    await wrapper.get('[data-testid="folder-submit"]').trigger('click')
    await flushPromises()

    expect(wrapper.get<HTMLButtonElement>('button[type="button"]').element.disabled).toBe(true)
  })

  it('emits busy while a submit is in flight, for a host that embeds this field inline', async () => {
    checkFolder.mockResolvedValue({
      resolvedPath: '/a',
      exists: true,
      isDirectory: true,
      writable: true,
      hasDatabase: false,
      usable: true
    })
    addDatabase.mockImplementation(() => new Promise(() => {}))
    const wrapper = mount(DatabaseFolderField)

    expect(wrapper.emitted('busy')?.at(-1)).toEqual([false])

    await typePath(wrapper, '/a')
    await wrapper.get('[data-testid="folder-submit"]').trigger('click')
    await flushPromises()

    expect(wrapper.emitted('busy')?.at(-1)).toEqual([true])
  })

  it('submits and shows the restarting overlay once a usable folder is confirmed', async () => {
    checkFolder.mockResolvedValue({
      resolvedPath: '/a',
      exists: true,
      isDirectory: true,
      writable: true,
      hasDatabase: false,
      usable: true
    })
    addDatabase.mockResolvedValue({
      mode: 'created',
      entry: { id: '1', label: 'a', path: '/a', active: true, reachable: true, addedAt: 't', lastUsedAt: 't' }
    })
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true }))
    vi.stubGlobal('location', { ...window.location, reload: vi.fn() })
    const wrapper = mount(DatabaseFolderField)

    await typePath(wrapper, '/a')
    await wrapper.get('[data-testid="folder-submit"]').trigger('click')
    await vi.runAllTimersAsync()
    await flushPromises()

    expect(addDatabase).toHaveBeenCalledWith({ path: '/a', label: undefined })
    expect(wrapper.find('[data-testid="restarting-overlay"]').exists()).toBe(true)
  })

  /**
   * The macOS application installs window.rekallDesktop before the page runs. Nothing else about
   * the field changes: the panel produces a string, and that string goes through the same check
   * and the same submit as a typed one.
   */
  describe('inside the macOS application', () => {
    it('fills the field with the folder chosen in the system panel', async () => {
      const pickFolder = vi.fn().mockResolvedValue('/Users/you/rekall')
      vi.stubGlobal('rekallDesktop', { pickFolder })
      checkFolder.mockResolvedValue({
        resolvedPath: '/Users/you/rekall',
        exists: true,
        isDirectory: true,
        writable: true,
        hasDatabase: true,
        usable: true
      })
      const wrapper = mount(DatabaseFolderField)

      await wrapper.get('[data-testid="folder-browse"]').trigger('click')
      await flushPromises()
      await vi.advanceTimersByTimeAsync(350)

      expect(pickFolder).toHaveBeenCalledWith('')
      expect(wrapper.get<HTMLInputElement>('[data-testid="database-folder-input"]').element.value).toBe(
        '/Users/you/rekall'
      )
      expect(wrapper.get('[data-testid="folder-submit"]').text()).toBe('Open this database')
    })

    it('opens the panel on what is already typed, and keeps it when the panel is dismissed', async () => {
      const pickFolder = vi.fn().mockResolvedValue(null)
      vi.stubGlobal('rekallDesktop', { pickFolder })
      checkFolder.mockResolvedValue({
        resolvedPath: '/a',
        exists: true,
        isDirectory: true,
        writable: true,
        hasDatabase: false,
        usable: true
      })
      const wrapper = mount(DatabaseFolderField)

      await typePath(wrapper, '/a')
      await wrapper.get('[data-testid="folder-browse"]').trigger('click')
      await flushPromises()

      expect(pickFolder).toHaveBeenCalledWith('/a')
      expect(wrapper.get<HTMLInputElement>('[data-testid="database-folder-input"]').element.value).toBe('/a')
      expect(wrapper.get('[data-testid="folder-submit"]').text()).toBe('Create database here')
    })

    it('survives a bridge that throws, because a broken panel must not break typing', async () => {
      vi.stubGlobal('rekallDesktop', {
        pickFolder: vi.fn().mockRejectedValue(new Error('no window'))
      })
      const wrapper = mount(DatabaseFolderField)

      await wrapper.get('[data-testid="folder-browse"]').trigger('click')
      await flushPromises()

      expect(wrapper.get<HTMLInputElement>('[data-testid="database-folder-input"]').element.value).toBe('')
    })
  })

  it('leaves the folder icon as decoration in a browser, where no picker can exist', () => {
    const wrapper = mount(DatabaseFolderField)

    expect(wrapper.get('[data-testid="folder-browse"]').element.tagName).toBe('SPAN')
  })
})
