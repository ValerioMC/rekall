import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/api/project-components.api'
import { useProjectComponentsStore } from '@/stores/projectComponents.store'
import type { ProjectComponent } from '@/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

vi.mock('@/api/project-components.api')

const PROJECT = 'p-1' as ProjectId

function component(id: string, position: number, name = id): ProjectComponent {
  return {
    id: id as ProjectComponentId,
    projectId: PROJECT,
    name,
    folder: `/work/${id}`,
    description: '',
    position,
    createdAt: '2026-10-08T10:00:00Z',
    updatedAt: '2026-10-08T10:00:00Z'
  }
}

describe('the project components store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('reads a project once opened and keeps the order the server gave', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue([component('a', 0), component('b', 1)])
    const store = useProjectComponentsStore()

    await store.load(PROJECT)

    expect(store.componentsOf(PROJECT).map((c) => c.id)).toEqual(['a', 'b'])
    expect(store.isLoading(PROJECT)).toBe(false)
  })

  it('adds a created component last and replaces an edited one in place', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue([component('a', 0)])
    vi.mocked(api.createProjectComponent).mockResolvedValue(component('b', 1))
    vi.mocked(api.updateProjectComponent).mockResolvedValue(component('a', 0, 'Renamed'))
    const store = useProjectComponentsStore()
    await store.load(PROJECT)

    await store.create(PROJECT, { folder: '/work/b' })
    await store.update('a' as ProjectComponentId, { name: 'Renamed' })

    expect(store.componentsOf(PROJECT).map((c) => c.name)).toEqual(['Renamed', 'b'])
  })

  it('shows a move at once and puts the list back when the server refuses it', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue([component('a', 0), component('b', 1)])
    vi.mocked(api.reorderProjectComponents).mockRejectedValue(new Error('refused'))
    const store = useProjectComponentsStore()
    await store.load(PROJECT)

    await expect(store.move(PROJECT, 'b' as ProjectComponentId, 'up')).rejects.toThrow('refused')

    expect(store.componentsOf(PROJECT).map((c) => c.id)).toEqual(['a', 'b'])
    expect(api.reorderProjectComponents).toHaveBeenCalledWith(PROJECT, ['b', 'a'])
  })

  it('takes the order the server answers a move with', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue([component('a', 0), component('b', 1)])
    vi.mocked(api.reorderProjectComponents).mockResolvedValue([component('b', 0), component('a', 1)])
    const store = useProjectComponentsStore()
    await store.load(PROJECT)

    await store.move(PROJECT, 'b' as ProjectComponentId, 'up')

    expect(store.componentsOf(PROJECT).map((c) => c.id)).toEqual(['b', 'a'])
  })

  it('drops a deleted component', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue([component('a', 0), component('b', 1)])
    vi.mocked(api.deleteProjectComponent).mockResolvedValue()
    const store = useProjectComponentsStore()
    await store.load(PROJECT)

    await store.remove(PROJECT, 'a' as ProjectComponentId)

    expect(store.componentsOf(PROJECT).map((c) => c.id)).toEqual(['b'])
  })

  it('holds nothing for a project it has not opened', () => {
    expect(useProjectComponentsStore().componentsOf('other' as ProjectId)).toEqual([])
  })

  it('tells a project with no components from one it has not fetched', async () => {
    vi.mocked(api.fetchProjectComponents).mockResolvedValue([])
    const store = useProjectComponentsStore()

    expect(store.isLoaded(PROJECT)).toBe(false)
    await store.ensureLoaded(PROJECT)
    await store.ensureLoaded(PROJECT)

    expect(store.isLoaded(PROJECT)).toBe(true)
    expect(api.fetchProjectComponents).toHaveBeenCalledTimes(1)
  })
})
