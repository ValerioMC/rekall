import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { clearExports, deleteExport, fetchExports } from '@/features/export/api/project-export.api'
import { useConsoleStore } from '@/stores/console.store'
import { asProjectExportId } from '@/model/branded'
import { entryOf } from './export-fixtures'

vi.mock('@/features/export/api/project-export.api', () => ({ fetchExports: vi.fn(), deleteExport: vi.fn(), clearExports: vi.fn() }))

const OLDER = entryOf({ id: asProjectExportId('e0000000-0000-4000-8000-000000000002'), summary: 'Older' })

describe('the exports slice', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(fetchExports).mockReset()
    vi.mocked(deleteExport).mockReset()
    vi.mocked(clearExports).mockReset()
  })

  it('loads the entries and says when', async () => {
    vi.mocked(fetchExports).mockResolvedValue([entryOf()])
    const store = useConsoleStore()

    expect(store.exportsLoaded).toBe(false)
    await store.loadExports()

    expect(store.exportEntries).toEqual([entryOf()])
    expect(store.exportsLoaded).toBe(true)
  })

  it('adds a new entry first and replaces one it already holds', () => {
    const store = useConsoleStore()
    store.addExport(OLDER)
    store.addExport(entryOf())
    store.addExport({ ...OLDER, summary: 'Changed' })

    expect(store.exportEntries.map((entry) => entry.summary)).toEqual(['Blueprint', 'Changed'])
  })

  it('removes an entry once the server has deleted it', async () => {
    vi.mocked(deleteExport).mockResolvedValue()
    const store = useConsoleStore()
    store.addExport(entryOf())

    await store.removeExport(entryOf().id)

    expect(deleteExport).toHaveBeenCalledWith(entryOf().id)
    expect(store.exportEntries).toEqual([])
  })

  it('keeps the entry when the server refuses the delete', async () => {
    vi.mocked(deleteExport).mockRejectedValue(new Error('refused'))
    const store = useConsoleStore()
    store.addExport(entryOf())

    await expect(store.removeExport(entryOf().id)).rejects.toThrow('refused')

    expect(store.exportEntries).toHaveLength(1)
  })

  it('reads the list again after a clear, since the server decides what it reached', async () => {
    vi.mocked(clearExports).mockResolvedValue(1)
    vi.mocked(fetchExports).mockResolvedValue([OLDER])
    const store = useConsoleStore()
    store.addExport(entryOf())

    const deleted = await store.clearExports({ status: 'FAILED' })

    expect(deleted).toBe(1)
    expect(clearExports).toHaveBeenCalledWith({ status: 'FAILED' })
    expect(store.exportEntries).toEqual([OLDER])
  })
})
