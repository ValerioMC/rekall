import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/api/project-sections.api'
import { useProjectSectionsStore } from '@/stores/projectSections.store'
import type { ProjectSection } from '@/model/project-section'
import type { ProjectId, ProjectSectionId } from '@/model/branded'

vi.mock('@/api/project-sections.api')

const PROJECT = 'p-1' as ProjectId

function section(id: string, position: number, title = id): ProjectSection {
  return {
    id: id as ProjectSectionId,
    projectId: PROJECT,
    title,
    bodyMarkdown: '',
    position,
    createdAt: '2026-10-05T10:00:00Z',
    updatedAt: '2026-10-05T10:00:00Z'
  }
}

describe('the project sections store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('reads a project once opened and keeps the order the server gave', async () => {
    vi.mocked(api.fetchProjectSections).mockResolvedValue([section('a', 0), section('b', 1)])
    const store = useProjectSectionsStore()
    expect(store.isLoaded(PROJECT)).toBe(false)

    await store.load(PROJECT)

    expect(store.sectionsOf(PROJECT).map((s) => s.id)).toEqual(['a', 'b'])
    expect(store.isLoaded(PROJECT)).toBe(true)
    expect(store.isLoading(PROJECT)).toBe(false)
  })

  it('adds a created section last and replaces an edited one in place', async () => {
    vi.mocked(api.fetchProjectSections).mockResolvedValue([section('a', 0)])
    vi.mocked(api.createProjectSection).mockResolvedValue(section('b', 1))
    vi.mocked(api.updateProjectSection).mockResolvedValue(section('a', 0, 'Renamed'))
    const store = useProjectSectionsStore()
    await store.load(PROJECT)

    await store.create(PROJECT, 'b')
    await store.update('a' as ProjectSectionId, { title: 'Renamed' })

    expect(store.sectionsOf(PROJECT).map((s) => s.title)).toEqual(['Renamed', 'b'])
  })

  it('shows a move at once and puts the list back when the server refuses it', async () => {
    vi.mocked(api.fetchProjectSections).mockResolvedValue([section('a', 0), section('b', 1)])
    vi.mocked(api.reorderProjectSections).mockRejectedValue(new Error('refused'))
    const store = useProjectSectionsStore()
    await store.load(PROJECT)

    await expect(store.move(PROJECT, 'b' as ProjectSectionId, 'up')).rejects.toThrow('refused')

    expect(store.sectionsOf(PROJECT).map((s) => s.id)).toEqual(['a', 'b'])
    expect(api.reorderProjectSections).toHaveBeenCalledWith(PROJECT, ['b', 'a'])
  })

  it('drops a deleted section and reads only the projects it already holds when told of a change', async () => {
    vi.mocked(api.fetchProjectSections).mockResolvedValue([section('a', 0), section('b', 1)])
    const store = useProjectSectionsStore()
    await store.applyEvent('other' as ProjectId)
    expect(api.fetchProjectSections).not.toHaveBeenCalled()
    await store.load(PROJECT)

    await store.remove(PROJECT, 'a' as ProjectSectionId)
    expect(store.sectionsOf(PROJECT).map((s) => s.id)).toEqual(['b'])

    await store.applyEvent(PROJECT)
    expect(api.fetchProjectSections).toHaveBeenCalledTimes(2)
  })
})
