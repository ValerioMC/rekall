import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount, type VueWrapper } from '@vue/test-utils'
import ProjectAboutGroup from '@/components/project/ProjectAboutGroup.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Project } from '@/model/catalog'
import type { ProjectId } from '@/model/branded'

const project = (id: string, description: string | null): Project =>
  ({ id: id as ProjectId, title: id, description }) as unknown as Project

let wrapper: VueWrapper | null = null
let save: ReturnType<typeof vi.fn>

function render(initial: Project): VueWrapper {
  wrapper = mount(ProjectAboutGroup, { props: { project: initial } })
  return wrapper
}

beforeEach(() => {
  vi.useFakeTimers()
  setActivePinia(createPinia())
  save = vi.fn().mockResolvedValue(undefined)
  useConsoleStore().saveProjectDescription = save as never
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  vi.useRealTimers()
})

describe('the about group', () => {
  it('shows the saved description', () => {
    const group = render(project('p-1', 'Billing platform.'))

    expect((group.get('[data-testid="project-description"]').element as HTMLTextAreaElement).value).toBe('Billing platform.')
  })

  it('saves after a pause in typing, once', async () => {
    const group = render(project('p-1', null))

    await group.get('[data-testid="project-description"]').setValue('New')
    await group.get('[data-testid="project-description"]').setValue('Newer')
    expect(save).not.toHaveBeenCalled()
    vi.advanceTimersByTime(700)

    expect(save).toHaveBeenCalledOnce()
    expect(save).toHaveBeenCalledWith('p-1', 'Newer')
  })

  it('saves a pending edit to the project it was typed for when another one opens', async () => {
    const group = render(project('p-1', null))
    await group.get('[data-testid="project-description"]').setValue('Typed for one')

    await group.setProps({ project: project('p-2', 'Two') })

    expect(save).toHaveBeenCalledWith('p-1', 'Typed for one')
    expect((group.get('[data-testid="project-description"]').element as HTMLTextAreaElement).value).toBe('Two')
  })

  it('saves a pending edit when it goes away', async () => {
    const group = render(project('p-1', null))
    await group.get('[data-testid="project-description"]').setValue('Last words')

    group.unmount()
    wrapper = null

    expect(save).toHaveBeenCalledWith('p-1', 'Last words')
  })
})
