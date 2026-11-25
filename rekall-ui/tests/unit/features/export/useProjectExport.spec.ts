import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiError } from '@/api/client'
import { useProjectExport, type ExportPhase } from '@/features/export/composables/useProjectExport'
import { drawDiagramImages } from '@/features/export/lib/diagram-images'
import { exportProject } from '@/features/export/api/project-export.api'
import { useConsoleStore } from '@/stores/console.store'
import type { ExportSelection } from '@/features/export/model/project-export'
import { COMPONENT, entryOf, FLOW, PAGE, PROJECT } from './export-fixtures'

vi.mock('@/features/export/api/project-export.api', () => ({ exportProject: vi.fn(), exportFileUrl: (id: string) => `/api/exports/${id}/file`, fetchExports: vi.fn(), deleteExport: vi.fn() }))
vi.mock('@/features/export/lib/diagram-images', async (original) => ({ ...(await original<object>()), drawDiagramImages: vi.fn() }))
vi.mock('@/features/project/store/projectSections.store', () => ({ useProjectSectionsStore: () => ({ sectionsOf: () => [PAGE] }) }))
vi.mock('@/features/project/store/projectComponents.store', () => ({ useProjectComponentsStore: () => ({ componentsOf: () => [COMPONENT] }) }))

const NOTHING: ExportSelection = { blueprint: false, sectionIds: [], componentIds: [] }
const BLUEPRINT: ExportSelection = { ...NOTHING, blueprint: true }

describe('useProjectExport', () => {
  const clicked: HTMLAnchorElement[] = []

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(exportProject).mockReset()
    vi.mocked(drawDiagramImages).mockReset()
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(function (this: HTMLAnchorElement) {
      clicked.push(this)
    })
  })

  afterEach(() => {
    clicked.length = 0
    vi.restoreAllMocks()
  })

  it('refuses to start while nothing is ticked', async () => {
    const exporting = useProjectExport(PROJECT, NOTHING)

    await exporting.start()

    expect(exportProject).not.toHaveBeenCalled()
    expect(exporting.phase.value).toEqual({ kind: 'idle' })
  })

  it('goes through drawing and writing, adds the entry and starts the download by link', async () => {
    const exporting = useProjectExport(PROJECT, BLUEPRINT)
    const seen: string[] = []
    vi.mocked(drawDiagramImages).mockImplementation(async (sources, _signal, onProgress) => {
      seen.push(exporting.phase.value.kind)
      onProgress({ finished: 1, total: sources.length, failures: [] })
      return [{ source: FLOW, png: new Uint8Array([1]) }]
    })
    vi.mocked(exportProject).mockImplementation(async () => {
      seen.push(exporting.phase.value.kind)
      return entryOf()
    })

    await exporting.start()

    expect(seen).toEqual(['drawing', 'writing'])
    expect(exporting.phase.value).toEqual({ kind: 'done', entry: entryOf() })
    expect(useConsoleStore().exportEntries).toEqual([entryOf()])
    expect(clicked.map((link) => link.getAttribute('href'))).toEqual([`/api/exports/${entryOf().id}/file`])
    expect(vi.mocked(exportProject).mock.calls[0]?.[1]).toMatchObject({ format: 'PDF', blueprint: true })
  })

  it('skips drawing when the ticked items hold no diagram', async () => {
    const exporting = useProjectExport(PROJECT, { ...NOTHING, componentIds: [COMPONENT.id] })
    vi.mocked(exportProject).mockResolvedValue(entryOf({ format: 'DOCX' }))

    await exporting.start()

    expect(drawDiagramImages).not.toHaveBeenCalled()
    expect(exporting.phase.value.kind).toBe('done')
  })

  it('keeps the selection and the server message when the export fails, recorded for a server error', async () => {
    const exporting = useProjectExport(PROJECT, { ...BLUEPRINT, sectionIds: [PAGE.id] })
    vi.mocked(drawDiagramImages).mockResolvedValue([])
    vi.mocked(exportProject).mockRejectedValue(new ApiError(500, 'Render error. The PDF could not be written: x', ''))

    await exporting.start()

    expect(exporting.phase.value).toEqual<ExportPhase>({ kind: 'failed', message: 'Render error. The PDF could not be written: x', recorded: true })
    expect(exporting.ticked.value).toBe(2)
    expect(useConsoleStore().exportEntries).toEqual([])
  })

  it('does not record a refusal or a browser failure', async () => {
    const exporting = useProjectExport(PROJECT, BLUEPRINT)
    vi.mocked(drawDiagramImages).mockRejectedValue(new Error('The canvas gave no image.'))

    await exporting.start()

    expect(exporting.phase.value).toEqual({ kind: 'failed', message: 'The canvas gave no image.', recorded: false })
  })

  it('goes back to idle with the selection kept when the drawing is cancelled', async () => {
    const exporting = useProjectExport(PROJECT, BLUEPRINT)
    vi.mocked(drawDiagramImages).mockImplementation((_sources, signal) => new Promise((_resolve, reject) => signal.addEventListener('abort', () => reject(signal.reason))))

    const running = exporting.start()
    exporting.cancel()
    await running

    expect(exporting.phase.value).toEqual({ kind: 'idle' })
    expect(exporting.ticked.value).toBe(1)
    expect(exportProject).not.toHaveBeenCalled()
  })

  it('ticks and unticks pages and components, singly and by group', () => {
    const exporting = useProjectExport(PROJECT, NOTHING)

    exporting.toggleSection(PAGE.id)
    exporting.setComponents([COMPONENT.id], true)
    expect(exporting.ticked.value).toBe(2)

    exporting.toggleSection(PAGE.id)
    exporting.setComponents([COMPONENT.id], false)
    expect(exporting.ticked.value).toBe(0)
  })
})
