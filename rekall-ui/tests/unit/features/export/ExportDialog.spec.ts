import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiError } from '@/api/client'
import ExportDialog from '@/features/export/components/ExportDialog.vue'
import { exportProject } from '@/features/export/api/project-export.api'
import { formatFileSize } from '@/common/format/file-size'
import type { ExportSelection } from '@/features/export/model/project-export'
import { COMPONENT, CONTEXT, entryOf, PAGE, PROJECT } from './export-fixtures'

vi.mock('@/features/export/api/project-export.api', () => ({ exportProject: vi.fn(), exportFileUrl: (id: string) => `/api/exports/${id}/file`, fetchExports: vi.fn(), deleteExport: vi.fn() }))
vi.mock('@/features/export/lib/diagram-images', async (original) => ({ ...(await original<object>()), drawDiagramImages: vi.fn(async () => []) }))
vi.mock('@/features/project/store/projectSections.store', () => ({ useProjectSectionsStore: () => ({ sectionsOf: () => [PAGE], contextsOf: () => [CONTEXT], isLoaded: () => true, load: vi.fn() }) }))
vi.mock('@/features/project/store/projectComponents.store', () => ({ useProjectComponentsStore: () => ({ componentsOf: () => [COMPONENT], ensureLoaded: vi.fn() }) }))

const NOTHING: ExportSelection = { blueprint: false, sectionIds: [], componentIds: [] }

function mountDialog(initial = NOTHING) {
  return mount(ExportDialog, {
    props: { project: PROJECT, initial },
    global: { stubs: { RouterLink: { template: '<a><slot /></a>' } } },
    attachTo: document.body
  })
}

describe('ExportDialog', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(exportProject).mockReset()
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => undefined)
  })

  afterEach(() => {
    vi.restoreAllMocks()
    document.body.innerHTML = ''
  })

  it('disables Export and says why while nothing is ticked', () => {
    const wrapper = mountDialog()

    expect(wrapper.get('[data-testid="export-submit"]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-testid="export-status"]').text()).toBe('Tick at least one item.')
  })

  it('opens with the current item ticked and enables Export', async () => {
    const wrapper = mountDialog({ ...NOTHING, sectionIds: [PAGE.id] })

    expect(wrapper.get('[data-testid="export-submit"]').attributes('disabled')).toBeUndefined()
    expect(wrapper.get('[data-testid="export-count"]').text()).toBe('1 ticked')
    expect(wrapper.get('[data-testid="export-status"]').text()).toBe('1 item, PDF')

    await wrapper.get('[data-testid="export-components-all"]').trigger('click')
    expect(wrapper.get('[data-testid="export-count"]').text()).toBe('2 ticked')
  })

  it('keeps the selection and shows the server message with where it was recorded when the export fails', async () => {
    vi.mocked(exportProject).mockRejectedValue(new ApiError(500, 'Render error. The PDF could not be written: x', ''))
    const wrapper = mountDialog({ ...NOTHING, sectionIds: [PAGE.id] })

    await wrapper.get('[data-testid="export-submit"]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="export-failed"]').text()).toContain('Render error. The PDF could not be written: x')
    expect(wrapper.get('[data-testid="export-failed"]').text()).toContain('Recorded on the Exports page.')
    expect(wrapper.get('[data-testid="export-submit"]').text()).toBe('Try again')
    expect(wrapper.get('[data-testid="export-count"]').text()).toBe('1 ticked')
  })

  it('shows the file and the diagrams that could not be drawn once it is done', async () => {
    vi.mocked(exportProject).mockResolvedValue(entryOf({ diagramsMissing: 2 }))
    const wrapper = mountDialog({ ...NOTHING, sectionIds: [PAGE.id] })

    await wrapper.get('[data-testid="export-submit"]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="export-done"]').text()).toContain('vega-blueprint.pdf')
    expect(wrapper.get('[data-testid="export-diagrams-missing"]').text()).toContain('2 diagrams could not be drawn')
    expect(wrapper.get('[data-testid="export-again"]').attributes('href')).toBe(`/api/exports/${entryOf().id}/file`)
  })
})

describe('formatFileSize', () => {
  it('writes bytes in the largest whole unit', () => {
    expect([formatFileSize(512), formatFileSize(188_416), formatFileSize(1_572_864)]).toEqual(['512 B', '184 KB', '1.5 MB'])
  })
})
