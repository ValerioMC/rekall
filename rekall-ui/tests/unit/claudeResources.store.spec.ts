import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { ApiError } from '@/api/client'
import { useClaudeResourcesStore } from '@/stores/claudeResources.store'
import { catalog, file } from './claude-resources.fixtures'

const api = vi.hoisted(() => ({
  fetchResourceCatalog: vi.fn(),
  fetchResourceFile: vi.fn(),
  saveResourceFile: vi.fn(),
  deleteResourceFile: vi.fn()
}))
vi.mock('@/api/claudeResources.api', () => api)

describe('the Claude resources store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    api.fetchResourceCatalog.mockResolvedValue(catalog())
  })

  it('opens a file into an editable draft that is not dirty until it is typed in', async () => {
    api.fetchResourceFile.mockResolvedValue(file('CLAUDE.md', 'one'))
    const store = useClaudeResourcesStore()

    await store.open('CLAUDE.md')
    expect(store.draft).toBe('one')
    expect(store.dirty).toBe(false)

    store.draft = 'two'
    expect(store.dirty).toBe(true)
    store.revert()
    expect(store.dirty).toBe(false)
  })

  it('saves from the version it was opened with and adopts the new one', async () => {
    api.fetchResourceFile.mockResolvedValue(file('CLAUDE.md', 'one'))
    api.saveResourceFile.mockResolvedValue(file('CLAUDE.md', 'two', { modifiedAt: '2026-10-12T10:00:00.000Z' }))
    const store = useClaudeResourcesStore()
    await store.open('CLAUDE.md')
    store.draft = 'two'

    expect(await store.save()).toBe(true)

    expect(api.saveResourceFile).toHaveBeenCalledWith('CLAUDE.md', 'two', '2026-10-12T09:00:00.000Z')
    expect(store.file?.summary.modifiedAt).toBe('2026-10-12T10:00:00.000Z')
    expect(store.dirty).toBe(false)
    expect(api.fetchResourceCatalog).toHaveBeenCalled()
  })

  it('keeps the draft and raises a conflict when the file changed on disk', async () => {
    api.fetchResourceFile.mockResolvedValue(file('CLAUDE.md', 'one'))
    api.saveResourceFile.mockRejectedValue(new ApiError(409, 'changed on disk', ''))
    const store = useClaudeResourcesStore()
    await store.open('CLAUDE.md')
    store.draft = 'mine'

    expect(await store.save()).toBe(false)

    expect(store.conflict).toBe(true)
    expect(store.draft).toBe('mine')
  })

  it('throws any other failure so the person is told', async () => {
    api.fetchResourceFile.mockResolvedValue(file('CLAUDE.md', 'one'))
    api.saveResourceFile.mockRejectedValue(new ApiError(500, 'disk full', ''))
    const store = useClaudeResourcesStore()
    await store.open('CLAUDE.md')
    store.draft = 'mine'

    await expect(store.save()).rejects.toThrow('disk full')
    expect(store.conflict).toBe(false)
  })

  it('overwrites with the version now on disk, keeping the draft', async () => {
    api.fetchResourceFile
      .mockResolvedValueOnce(file('CLAUDE.md', 'one'))
      .mockResolvedValueOnce(file('CLAUDE.md', 'by a session', { modifiedAt: '2026-10-12T09:30:00.000Z' }))
    api.saveResourceFile
      .mockRejectedValueOnce(new ApiError(409, 'changed', ''))
      .mockResolvedValueOnce(file('CLAUDE.md', 'mine', { modifiedAt: '2026-10-12T10:00:00.000Z' }))
    const store = useClaudeResourcesStore()
    await store.open('CLAUDE.md')
    store.draft = 'mine'
    await store.save()

    await store.overwrite()

    expect(api.saveResourceFile).toHaveBeenLastCalledWith('CLAUDE.md', 'mine', '2026-10-12T09:30:00.000Z')
    expect(store.conflict).toBe(false)
    expect(store.file?.body).toBe('mine')
  })

  it('drops the draft for the disk version on reload', async () => {
    api.fetchResourceFile
      .mockResolvedValueOnce(file('CLAUDE.md', 'one'))
      .mockResolvedValueOnce(file('CLAUDE.md', 'by a session'))
    const store = useClaudeResourcesStore()
    await store.open('CLAUDE.md')
    store.draft = 'mine'

    await store.reloadFromDisk()

    expect(store.draft).toBe('by a session')
    expect(store.conflict).toBe(false)
  })

  it('creates a file that must not exist yet and opens it', async () => {
    api.saveResourceFile.mockResolvedValue(file('commands/new.md', '---\n---\n'))
    const store = useClaudeResourcesStore()

    await store.create('commands/new.md', '---\n---\n')

    expect(api.saveResourceFile).toHaveBeenCalledWith('commands/new.md', '---\n---\n', null)
    expect(store.selectedPath).toBe('commands/new.md')
  })

  it('closes the open file when it is the one deleted', async () => {
    api.fetchResourceFile.mockResolvedValue(file('memory/a.md', 'x'))
    api.deleteResourceFile.mockResolvedValue(undefined)
    const store = useClaudeResourcesStore()
    await store.open('memory/a.md')

    await store.remove('memory/a.md')

    expect(store.file).toBeNull()
    expect(api.deleteResourceFile).toHaveBeenCalledWith('memory/a.md')
  })

  it('keeps the open file when another one is deleted', async () => {
    api.fetchResourceFile.mockResolvedValue(file('memory/a.md', 'x'))
    api.deleteResourceFile.mockResolvedValue(undefined)
    const store = useClaudeResourcesStore()
    await store.open('memory/a.md')

    await store.remove('memory/b.md')

    expect(store.selectedPath).toBe('memory/a.md')
  })
})
