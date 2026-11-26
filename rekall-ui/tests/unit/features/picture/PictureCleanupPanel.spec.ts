import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, disposePinia, setActivePinia, type Pinia } from 'pinia'
import { enableAutoUnmount, flushPromises, mount } from '@vue/test-utils'
import PictureCleanupPanel from '@/features/picture/components/PictureCleanupPanel.vue'
import type { PictureCleanupStatus } from '@/features/picture/model/picture-cleanup'
import { fakeCleanupServer } from '../../support/fakeCleanupServer'

const unused = { url: '/api/images/acme/web/web-12/old-mock-77aa01de.png', folder: 'acme/web/web-12', sizeBytes: 245760 }
const listed: PictureCleanupStatus = { state: 'READY', pictures: [unused], totalBytes: unused.sizeBytes }

/** Every observed box is in view at once. */
class SeeingObserver {
  constructor(private readonly report: (entries: { isIntersecting: boolean }[]) => void) {}
  observe(): void {
    this.report([{ isIntersecting: true }])
  }
  disconnect(): void {}
}

describe('the unused images panel', () => {
  let pinia: Pinia

  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    document.body.innerHTML = ''
  })

  enableAutoUnmount(afterEach)

  afterEach(() => {
    disposePinia(pinia)
    vi.unstubAllGlobals()
  })

  async function mountPanel() {
    const panel = mount(PictureCleanupPanel, { attachTo: document.body, global: { plugins: [pinia] } })
    await flushPromises()
    return panel
  }

  const inDialog = (label: string) =>
    [...document.querySelectorAll<HTMLButtonElement>('[role="alertdialog"] button')].find((button) => button.textContent?.trim() === label)!

  it('says what it finds while idle, and shows the progress of the search it starts', async () => {
    const server = fakeCleanupServer()
    server.afterRequest['POST /api/image-cleanup'] = { state: 'SEARCHING', percentage: 0 }
    const panel = await mountPanel()
    expect(panel.text()).toContain('Images written in the last 24 hours are left alone.')

    await panel.get('[data-testid="picture-cleanup-search"]').trigger('click')
    await flushPromises()
    server.emit({ state: 'SEARCHING', percentage: 42, count: 0 })
    await flushPromises()

    expect(server.requests).toContain('POST /api/image-cleanup')
    expect(panel.get('[role="progressbar"]').attributes('aria-valuenow')).toBe('42')
  })

  it('shows where a run from before a reload stands', async () => {
    fakeCleanupServer({ state: 'DELETING', percentage: 70 })

    const panel = await mountPanel()

    expect(panel.get('[role="progressbar"]').attributes('aria-label')).toBe('Deleting unused images')
    expect(panel.get('[role="progressbar"]').attributes('aria-valuenow')).toBe('70')
  })

  it('previews each unused picture with its thumbnail, folder, size and the total', async () => {
    vi.stubGlobal('IntersectionObserver', SeeingObserver)
    fakeCleanupServer(listed)

    const panel = await mountPanel()

    const row = panel.get('[data-testid="unused-picture"]')
    expect(row.get('img').attributes('src')).toBe(unused.url)
    expect(row.text()).toContain('old-mock-77aa01de.png')
    expect(row.text()).toContain('acme/web/web-12')
    expect(row.text()).toContain('240 KB')
    expect(panel.text()).toContain('1 image no text shows, 240 KB')
    expect(panel.get('[data-testid="picture-cleanup-delete"]').text()).toBe('Delete 1 image')
  })

  it('deletes only after the confirm that says the files go for good', async () => {
    const server = fakeCleanupServer(listed)
    server.afterRequest['POST /api/image-cleanup/deletion'] = { state: 'FINISHED', removed: 1, freedBytes: 245760, kept: 0, notDeleted: 0 }
    const panel = await mountPanel()

    await panel.get('[data-testid="picture-cleanup-delete"]').trigger('click')
    expect(document.body.textContent).toContain('deleted for good')
    inDialog('Keep it').click()
    await flushPromises()
    expect(server.requests).toEqual(['GET /api/image-cleanup'])

    await panel.get('[data-testid="picture-cleanup-delete"]').trigger('click')
    inDialog('Delete 1 image').click()
    await flushPromises()

    expect(server.requests).toContain('POST /api/image-cleanup/deletion')
    expect(panel.text()).toContain('1 removed, 240 KB freed')
  })

  it('cancels the list without deleting', async () => {
    const server = fakeCleanupServer(listed)
    server.afterRequest['DELETE /api/image-cleanup'] = { state: 'IDLE' }
    const panel = await mountPanel()

    await panel.get('[data-testid="picture-cleanup-cancel"]').trigger('click')
    await flushPromises()

    expect(server.requests).toEqual(['GET /api/image-cleanup', 'DELETE /api/image-cleanup', 'GET /api/image-cleanup'])
    expect(panel.find('[data-testid="unused-picture"]').exists()).toBe(false)
    expect(panel.text()).toContain('Find unused images')
  })

  it('says nothing is unused for an empty result', async () => {
    fakeCleanupServer({ state: 'READY', pictures: [], totalBytes: 0 })

    const panel = await mountPanel()

    expect(panel.text()).toContain('No unused images found')
    expect(panel.get('[data-testid="picture-cleanup-search"]').text()).toBe('Search again')
    expect(panel.find('[data-testid="picture-cleanup-delete"]').exists()).toBe(false)
  })

  it('says what a finished run left behind', async () => {
    fakeCleanupServer({ state: 'FINISHED', removed: 2, freedBytes: 2048, kept: 1, notDeleted: 3 })

    const panel = await mountPanel()

    expect(panel.text()).toContain('2 removed, 2 KB freed')
    expect(panel.text()).toContain('1 kept')
    expect(panel.text()).toContain('3 could not be deleted')
  })

  it('shows why a run failed', async () => {
    fakeCleanupServer({ state: 'FAILED', message: 'database is locked' })

    const panel = await mountPanel()

    expect(panel.get('[role="alert"]').text()).toContain('database is locked')
  })

  it('offers another read when the status cannot be read', async () => {
    const server = fakeCleanupServer()
    server.unreadable = true
    const panel = await mountPanel()
    expect(panel.text()).toContain("Couldn't read")

    server.unreadable = false
    await panel.get('button').trigger('click')
    await flushPromises()

    expect(panel.text()).toContain('Find unused images')
  })
})
