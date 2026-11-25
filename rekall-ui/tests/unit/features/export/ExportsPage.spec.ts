import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import ExportsPage from '@/features/export/components/ExportsPage.vue'
import { clearExports, deleteExport, fetchExports } from '@/features/export/api/project-export.api'
import { asCompanyId, asProjectExportId, asProjectId } from '@/model/branded'
import { entryOf } from './export-fixtures'

vi.mock('@/features/export/api/project-export.api', () => ({
  fetchExports: vi.fn(),
  deleteExport: vi.fn(),
  clearExports: vi.fn(),
  exportFileUrl: (id: string) => `/api/exports/${id}/file`,
  exportProject: vi.fn()
}))

const DONE = entryOf({ summary: 'Blueprint, 3 pages' })
const WITH_MISSING = entryOf({ id: asProjectExportId('e0000000-0000-4000-8000-000000000002'), summary: 'Layers', diagramsMissing: 2, createdAt: '2026-11-25T11:00:00Z' })
const FAILED = entryOf({
  id: asProjectExportId('e0000000-0000-4000-8000-000000000003'),
  summary: 'Pages',
  status: 'FAILED',
  failureKind: 'RENDER',
  failureMessage: 'Render error. The PDF could not be written: font subset failed.',
  sizeBytes: 0,
  createdAt: '2026-11-25T10:00:00Z'
})
const OTHER_COMPANY = entryOf({
  id: asProjectExportId('e0000000-0000-4000-8000-000000000004'),
  companyId: asCompanyId('d0000000-0000-4000-8000-000000000002'),
  companyName: 'Northwind',
  projectId: asProjectId('22222222-2222-4222-8222-222222222222'),
  projectLabel: 'kestrel',
  projectTitle: 'Kestrel Platform',
  summary: 'Everything',
  createdAt: '2026-11-24T10:00:00Z'
})

let wrapper: VueWrapper | null = null

async function openPage(entries = [DONE, WITH_MISSING, FAILED, OTHER_COMPANY]): Promise<VueWrapper> {
  vi.mocked(fetchExports).mockResolvedValue(entries)
  wrapper = mount(ExportsPage, {
    global: { stubs: { AppCatalogHeader: { template: '<div><slot name="actions" /></div>' }, LibrarySubNav: true, RouterLink: { template: '<a><slot /></a>' }, CopyAnchorButton: true, CompanyMark: true } },
    attachTo: document.body
  })
  await flushPromises()
  return wrapper
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.mocked(fetchExports).mockReset()
  vi.mocked(deleteExport).mockReset().mockResolvedValue()
  vi.mocked(clearExports).mockReset()
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  document.body.innerHTML = ''
})

describe('the Exports page', () => {
  it('lists every export under its company and project', async () => {
    const page = await openPage()

    expect(page.findAll('[data-testid^="export-company-toggle-"]').map((toggle) => toggle.text())).toEqual(['VFORGE', 'Northwind'])
    expect(page.findAll('[data-testid^="export-project-toggle-"]').map((toggle) => toggle.text())).toEqual(['Vega Platform', 'Kestrel Platform'])
    expect(page.findAll('li[data-testid^="export-row-"]')).toHaveLength(4)
  })

  it('shows a failed export apart, with its kind and message and no download', async () => {
    const page = await openPage()
    const row = page.get(`[data-testid="export-row-${FAILED.id}"]`)

    expect(row.attributes('data-status')).toBe('FAILED')
    expect(row.get('[data-testid="export-failure-kind"]').text()).toBe('RENDER')
    expect(row.get('[data-testid="export-failure-message"]').text()).toBe('Render error. The PDF could not be written: font subset failed.')
    expect(row.find('[data-testid="export-download"]').exists()).toBe(false)
    expect(page.get('[data-testid^="export-project-failed"]').text()).toBe('1 failed')
  })

  it('says on a done export that some diagrams could not be drawn', async () => {
    const page = await openPage()

    expect(page.get(`[data-testid="export-row-${WITH_MISSING.id}"] [data-testid="export-row-diagrams-missing"]`).text()).toContain('2 diagrams could not be drawn')
    expect(page.find(`[data-testid="export-row-${DONE.id}"] [data-testid="export-row-diagrams-missing"]`).exists()).toBe(false)
  })

  it('links Download to the file of a done export', async () => {
    const page = await openPage()

    expect(page.get(`[data-testid="export-row-${DONE.id}"] [data-testid="export-download"]`).attributes('href')).toBe(`/api/exports/${DONE.id}/file`)
  })

  it('asks before deleting a done export, and the row leaves once it is deleted', async () => {
    const page = await openPage()
    const row = (): ReturnType<typeof page.find> => page.find(`[data-testid="export-row-${DONE.id}"]`)

    await row().get('[data-testid="export-delete"]').trigger('click')
    expect(row().text()).toContain('Delete this file?')
    expect(deleteExport).not.toHaveBeenCalled()

    await row().get('[data-testid="export-keep"]').trigger('click')
    expect(row().text()).not.toContain('Delete this file?')

    await row().get('[data-testid="export-delete"]').trigger('click')
    await row().get('[data-testid="export-delete-confirm"]').trigger('click')
    await flushPromises()

    expect(deleteExport).toHaveBeenCalledWith(DONE.id)
    expect(row().exists()).toBe(false)
  })

  it('deletes a failed export at once, since nothing is lost but a message', async () => {
    const page = await openPage()

    await page.get(`[data-testid="export-row-${FAILED.id}"] [data-testid="export-delete"]`).trigger('click')
    await flushPromises()

    expect(deleteExport).toHaveBeenCalledWith(FAILED.id)
    expect(page.find(`[data-testid="export-row-${FAILED.id}"]`).exists()).toBe(false)
  })

  it('narrows the list with the status filter and says how much is shown', async () => {
    const page = await openPage()

    await page.get('[data-testid="segment-failed"]').trigger('click')

    expect(page.findAll('li[data-testid^="export-row-"]')).toHaveLength(1)
    expect(page.get('[data-testid="exports-showing"]').text()).toBe('Showing 1 of 4')
  })

  it('clears every failed export after a confirmation that names the count', async () => {
    vi.mocked(clearExports).mockResolvedValue(1)
    const page = await openPage()

    await page.get('[data-testid="export-clear-failed"]').trigger('click')
    expect(document.body.textContent).toContain('Delete 1 export?')
    vi.mocked(fetchExports).mockResolvedValue([DONE, WITH_MISSING, OTHER_COMPANY])
    const confirm = Array.from(document.body.querySelectorAll('button')).find((button) => button.textContent?.trim() === 'Delete 1 export')
    confirm?.click()
    await flushPromises()

    expect(clearExports).toHaveBeenCalledWith({ status: 'FAILED' })
    expect(page.find(`[data-testid="export-row-${FAILED.id}"]`).exists()).toBe(false)
  })

  it('tells what to do when nothing was ever exported', async () => {
    const page = await openPage([])

    expect(page.get('[data-testid="empty-state"]').text()).toContain('Nothing exported yet')
    expect(page.find('[data-testid="exports-toolbar"]').exists()).toBe(false)
  })

  it('says so when the filters leave nothing, and clears them', async () => {
    const page = await openPage()

    await page.get('input[type="search"]').setValue('no such export')
    expect(page.get('[data-testid="empty-state"]').text()).toContain('No export matches')

    await page.get('[data-testid="exports-clear-filters"]').trigger('click')
    expect(page.findAll('li[data-testid^="export-row-"]')).toHaveLength(4)
  })

  it('keeps the message of a list that could not be loaded, with a way to try again', async () => {
    vi.mocked(fetchExports).mockRejectedValue(new Error('The server did not answer'))
    wrapper = mount(ExportsPage, { global: { stubs: { AppCatalogHeader: true, LibrarySubNav: true } } })
    await flushPromises()

    expect(wrapper.get('[data-testid="exports-load-failed"]').text()).toContain('The server did not answer')
  })
})
