import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/project/api/project-sections.api'
import { openTerminal } from '@/features/claude/api/terminal.api'
import { useProjectSectionsStore } from '@/features/project/store/projectSections.store'
import type { ProjectSection } from '@/features/project/model/project-section'
import type { SectionContext } from '@/features/project/model/section-context'
import type { ProjectId, ProjectSectionId, SectionContextId } from '@/model/branded'

vi.mock('@/features/project/api/project-sections.api')
vi.mock('@/features/claude/api/terminal.api', () => ({
  openTerminal: vi.fn(async (target: unknown) => ({ id: 't-1', kind: 'TERMINAL', target })),
  fetchTerminals: vi.fn(),
  closeTerminal: vi.fn(),
  sendTerminalInput: vi.fn()
}))

const PROJECT = 'p-1' as ProjectId
const OPS = 'ops' as SectionContextId
const TEAM = 'team' as SectionContextId

function context(id: SectionContextId, position: number, title: string = id): SectionContext {
  return { id, projectId: PROJECT, title, position, createdAt: '2026-10-05T10:00:00Z', updatedAt: '2026-10-05T10:00:00Z' }
}

function section(id: string, position: number, title = id, contextId: SectionContextId = OPS): ProjectSection {
  return {
    id: id as ProjectSectionId,
    projectId: PROJECT,
    contextId,
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
    vi.mocked(api.fetchSectionContexts).mockResolvedValue([context(OPS, 0)])
    const store = useProjectSectionsStore()
    expect(store.isLoaded(PROJECT)).toBe(false)

    await store.load(PROJECT)

    expect(store.sectionsOf(PROJECT).map((s) => s.id)).toEqual(['a', 'b'])
    expect(store.contextsOf(PROJECT).map((c) => c.id)).toEqual([OPS])
    expect(store.isLoaded(PROJECT)).toBe(true)
    expect(store.isLoading(PROJECT)).toBe(false)
  })

  it('adds a created section last and replaces an edited one in place', async () => {
    vi.mocked(api.fetchProjectSections).mockResolvedValue([section('a', 0)])
    vi.mocked(api.fetchSectionContexts).mockResolvedValue([context(OPS, 0)])
    vi.mocked(api.createProjectSection).mockResolvedValue(section('b', 1))
    vi.mocked(api.updateProjectSection).mockResolvedValue(section('a', 0, 'Renamed'))
    const store = useProjectSectionsStore()
    await store.load(PROJECT)

    await store.create(PROJECT, 'b')
    await store.update('a' as ProjectSectionId, { title: 'Renamed' })

    expect(store.sectionsOf(PROJECT).map((s) => s.title)).toEqual(['Renamed', 'b'])
    expect(api.fetchSectionContexts).toHaveBeenCalledTimes(1)
  })

  it('creates a section with the text and the context it is given', async () => {
    vi.mocked(api.createProjectSection).mockResolvedValue(section('a', 0, 'a', TEAM))
    vi.mocked(api.fetchSectionContexts).mockResolvedValue([context(TEAM, 0)])
    const store = useProjectSectionsStore()

    await store.create(PROJECT, 'Reference', '# Reference', TEAM)

    expect(api.createProjectSection).toHaveBeenCalledWith(PROJECT, { title: 'Reference', bodyMarkdown: '# Reference', contextId: TEAM })
  })

  it('reads the contexts again when a created section landed in one the console has not seen', async () => {
    vi.mocked(api.createProjectSection).mockResolvedValue(section('a', 0, 'a', OPS))
    vi.mocked(api.fetchSectionContexts).mockResolvedValue([context(OPS, 0, 'General')])
    const store = useProjectSectionsStore()

    await store.create(PROJECT, 'Reference')

    expect(api.createProjectSection).toHaveBeenCalledWith(PROJECT, { title: 'Reference', bodyMarkdown: '' })
    expect(store.contextsOf(PROJECT).map((c) => c.title)).toEqual(['General'])
  })

  it('shows an arrangement at once and puts the old one back when the server refuses it', async () => {
    vi.mocked(api.fetchProjectSections).mockResolvedValue([section('a', 0), section('b', 1)])
    vi.mocked(api.fetchSectionContexts).mockResolvedValue([context(OPS, 0), context(TEAM, 1)])
    vi.mocked(api.arrangeProjectSections).mockRejectedValue(new Error('refused'))
    const store = useProjectSectionsStore()
    await store.load(PROJECT)
    const placements = [
      { contextId: TEAM, sectionIds: ['b' as ProjectSectionId] },
      { contextId: OPS, sectionIds: ['a' as ProjectSectionId] }
    ]

    await expect(store.arrange(PROJECT, placements)).rejects.toThrow('refused')

    expect(store.sectionsOf(PROJECT).map((s) => [s.id, s.contextId])).toEqual([['a', OPS], ['b', OPS]])
    expect(store.contextsOf(PROJECT).map((c) => c.id)).toEqual([OPS, TEAM])
    expect(api.arrangeProjectSections).toHaveBeenCalledWith(PROJECT, placements)
  })

  it('keeps what the server answers to an arrangement', async () => {
    vi.mocked(api.fetchProjectSections).mockResolvedValue([section('a', 0), section('b', 1)])
    vi.mocked(api.fetchSectionContexts).mockResolvedValue([context(OPS, 0), context(TEAM, 1)])
    vi.mocked(api.arrangeProjectSections).mockResolvedValue({
      contexts: [context(TEAM, 0), context(OPS, 1)],
      sections: [section('b', 0, 'b', TEAM), section('a', 0, 'a', OPS)]
    })
    const store = useProjectSectionsStore()
    await store.load(PROJECT)

    await store.arrange(PROJECT, [
      { contextId: TEAM, sectionIds: ['b' as ProjectSectionId] },
      { contextId: OPS, sectionIds: ['a' as ProjectSectionId] }
    ])

    expect(store.contextsOf(PROJECT).map((c) => c.id)).toEqual([TEAM, OPS])
    expect(store.sectionsOf(PROJECT).map((s) => [s.id, s.contextId])).toEqual([['b', TEAM], ['a', OPS]])
  })

  it('adds, renames and drops a context', async () => {
    vi.mocked(api.createSectionContext).mockResolvedValue(context(TEAM, 0, 'Team'))
    vi.mocked(api.renameSectionContext).mockResolvedValue(context(TEAM, 0, 'People'))
    vi.mocked(api.deleteSectionContext).mockResolvedValue()
    const store = useProjectSectionsStore()

    await store.createContext(PROJECT, 'Team')
    await store.renameContext(TEAM, 'People')
    expect(store.contextsOf(PROJECT).map((c) => c.title)).toEqual(['People'])

    await store.removeContext(PROJECT, TEAM)
    expect(store.contextsOf(PROJECT)).toEqual([])
  })

  it('drops a deleted section and reads only the projects it already holds when told of a change', async () => {
    vi.mocked(api.fetchProjectSections).mockResolvedValue([section('a', 0), section('b', 1)])
    vi.mocked(api.fetchSectionContexts).mockResolvedValue([context(OPS, 0)])
    const store = useProjectSectionsStore()
    await store.applyEvent('other' as ProjectId)
    expect(api.fetchProjectSections).not.toHaveBeenCalled()
    await store.load(PROJECT)

    await store.remove(PROJECT, 'a' as ProjectSectionId)
    expect(store.sectionsOf(PROJECT).map((s) => s.id)).toEqual(['b'])

    await store.applyEvent(PROJECT)
    expect(api.fetchProjectSections).toHaveBeenCalledTimes(2)
  })

  it('opens a reorganize session on the project alone', async () => {
    const store = useProjectSectionsStore()

    await store.reorganize(PROJECT)

    expect(openTerminal).toHaveBeenCalledWith({ projectId: PROJECT }, expect.objectContaining({ mode: 'REORGANIZE' }))
  })

  it('opens an autoconfigure session on the project alone', async () => {
    const store = useProjectSectionsStore()

    await store.autoconfigure(PROJECT)

    expect(openTerminal).toHaveBeenCalledWith({ projectId: PROJECT }, expect.objectContaining({ mode: 'AUTOCONFIGURE' }))
  })
})
