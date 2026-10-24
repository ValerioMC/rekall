import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import ProjectRepositoryGroup from '@/features/project/components/ProjectRepositoryGroup.vue'
import * as catalogApi from '@/features/catalog/api/catalog.api'
import { useConsoleStore } from '@/stores/console.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'

vi.mock('@/features/catalog/api/catalog.api', () => ({ fetchProjectRepository: vi.fn() }))

const project = (id: string, repoFolder: string | null, autoCommit = false): Project =>
  ({ id: id as ProjectId, title: id, repoFolder, autoCommit }) as unknown as Project

let wrapper: VueWrapper | null = null
let saveFolder: ReturnType<typeof vi.fn>
let saveAutoCommit: ReturnType<typeof vi.fn>

function render(initial: Project): VueWrapper {
  wrapper = mount(ProjectRepositoryGroup, { props: { project: initial } })
  return wrapper
}

beforeEach(() => {
  vi.useFakeTimers()
  setActivePinia(createPinia())
  vi.mocked(catalogApi.fetchProjectRepository).mockReset().mockResolvedValue({ repository: false } as never)
  const store = useConsoleStore()
  saveFolder = vi.fn().mockResolvedValue(undefined)
  saveAutoCommit = vi.fn().mockResolvedValue(undefined)
  store.saveProjectRepoFolder = saveFolder as never
  store.saveProjectAutoCommit = saveAutoCommit as never
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  vi.useRealTimers()
})

describe('the repository group', () => {
  it('shows the folder and asks git about it once', async () => {
    const group = render(project('p-1', '/work/vega'))
    await flushPromises()

    expect((group.get('[data-testid="project-repo-folder"]').element as HTMLInputElement).value).toBe('/work/vega')
    expect(catalogApi.fetchProjectRepository).toHaveBeenCalledOnce()
    expect(catalogApi.fetchProjectRepository).toHaveBeenCalledWith('p-1')
  })

  it('saves a typed folder after a pause, then asks git about it again', async () => {
    const group = render(project('p-1', null))
    await flushPromises()

    await group.get('[data-testid="project-repo-folder"]').setValue('/work/new')
    vi.advanceTimersByTime(700)
    await flushPromises()

    expect(saveFolder).toHaveBeenCalledWith('p-1', '/work/new')
    expect(catalogApi.fetchProjectRepository).toHaveBeenCalledTimes(2)
  })

  it('saves a pending folder to the project it was typed for when another one opens', async () => {
    const group = render(project('p-1', null))
    await group.get('[data-testid="project-repo-folder"]').setValue('/work/one')

    await group.setProps({ project: project('p-2', '/work/two') })
    await flushPromises()

    expect(saveFolder).toHaveBeenCalledWith('p-1', '/work/one')
    expect((group.get('[data-testid="project-repo-folder"]').element as HTMLInputElement).value).toBe('/work/two')
  })
})
