import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, disposePinia, setActivePinia, type Pinia } from 'pinia'
import { flushPromises } from '@vue/test-utils'
import { usePictureCleanupStore } from '@/features/picture/store/pictureCleanup.store'
import { useToastStore } from '@/stores/toast.store'
import { fakeCleanupServer } from '../../support/fakeCleanupServer'
import { installStorage } from '../../support/memoryStorage'

const unused = { url: '/api/images/acme/web/web-12/old-mock-77aa01de.png', folder: 'acme/web/web-12', sizeBytes: 245760 }

describe('following the image cleanup', () => {
  let pinia: Pinia
  const notify = vi.fn(async () => true)

  beforeEach(() => {
    installStorage()
    pinia = createPinia()
    setActivePinia(pinia)
    notify.mockClear()
    window.rekallDesktop = { notify } as unknown as typeof window.rekallDesktop
    vi.spyOn(document, 'hasFocus').mockReturnValue(true)
  })

  afterEach(() => {
    disposePinia(pinia)
    delete window.rekallDesktop
    vi.restoreAllMocks()
    vi.unstubAllGlobals()
  })

  const toasts = () => useToastStore().toasts.map(({ kind, message }) => ({ kind, message }))

  it('shows the percentage a frame carries, without reading again', () => {
    const server = fakeCleanupServer()
    const store = usePictureCleanupStore()

    server.emit({ state: 'SEARCHING', percentage: 42, count: 0 })

    expect(store.status).toEqual({ state: 'SEARCHING', percentage: 42 })
    expect(server.requests).toEqual([])
  })

  it('reads the list when a search ends and says so once', async () => {
    const server = fakeCleanupServer({ state: 'READY', pictures: [unused], totalBytes: unused.sizeBytes })
    const store = usePictureCleanupStore()

    server.emit({ state: 'READY', percentage: 100, count: 1 })
    await flushPromises()

    expect(store.status?.state).toBe('READY')
    expect(toasts()).toEqual([{ kind: 'success', message: '1 unused image found' }])
    expect(notify).not.toHaveBeenCalled()
  })

  it('says what a deletion removed and freed', async () => {
    const server = fakeCleanupServer({ state: 'FINISHED', removed: 1, freedBytes: 245760, kept: 0, notDeleted: 0 })
    usePictureCleanupStore()

    server.emit({ state: 'FINISHED', percentage: 100, count: 1 })
    await flushPromises()

    expect(toasts()).toEqual([{ kind: 'success', message: '1 removed, 240 KB freed' }])
  })

  it('tells a failure as an error', async () => {
    const server = fakeCleanupServer({ state: 'FAILED', message: 'database is locked' })
    usePictureCleanupStore()

    server.emit({ state: 'FAILED', percentage: 0, count: 0 })
    await flushPromises()

    expect(toasts()).toEqual([{ kind: 'error', message: 'Image cleanup failed: database is locked' }])
  })

  it('says nothing when a list is discarded', async () => {
    const server = fakeCleanupServer()
    usePictureCleanupStore()

    server.emit({ state: 'IDLE', percentage: 0, count: 0 })
    await flushPromises()

    expect(toasts()).toEqual([])
  })

  it('posts a system notice only while the console is in the background', async () => {
    vi.mocked(document.hasFocus).mockReturnValue(false)
    const server = fakeCleanupServer({ state: 'READY', pictures: [], totalBytes: 0 })
    usePictureCleanupStore()

    server.emit({ state: 'READY', percentage: 100, count: 0 })
    await flushPromises()

    expect(notify.mock.calls).toEqual([[{ title: 'Search for unused images ended', body: 'No unused images found' }]])
  })

  it('keeps the newer frame when an earlier read answers late', async () => {
    const server = fakeCleanupServer({ state: 'SEARCHING', percentage: 3 })
    const store = usePictureCleanupStore()

    const reading = store.load()
    server.emit({ state: 'SEARCHING', percentage: 60, count: 0 })
    await reading

    expect(store.status).toEqual({ state: 'SEARCHING', percentage: 60 })
  })

  it('discards the listed result before searching again', async () => {
    const server = fakeCleanupServer({ state: 'READY', pictures: [], totalBytes: 0 })
    server.afterRequest['POST /api/image-cleanup'] = { state: 'SEARCHING', percentage: 0 }
    const store = usePictureCleanupStore()
    await store.load()

    await store.search()

    expect(server.requests).toEqual(['GET /api/image-cleanup', 'DELETE /api/image-cleanup', 'POST /api/image-cleanup', 'GET /api/image-cleanup'])
    expect(store.status).toEqual({ state: 'SEARCHING', percentage: 0 })
  })
})
