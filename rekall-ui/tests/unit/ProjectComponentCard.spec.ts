import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import ProjectComponentCard from '@/components/project/ProjectComponentCard.vue'
import * as api from '@/api/project-components.api'
import { useProjectComponentsStore } from '@/stores/projectComponents.store'
import type { ProjectComponent } from '@/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

vi.mock('@/api/project-components.api')

const PROJECT = 'p-1' as ProjectId

const saved: ProjectComponent = {
  id: 'c-1' as ProjectComponentId,
  projectId: PROJECT,
  name: 'API',
  folder: '/work/api',
  description: 'The REST surface.',
  position: 0,
  createdAt: '2026-10-08T10:00:00Z',
  updatedAt: '2026-10-08T10:00:00Z'
}

let wrapper: VueWrapper | null = null

async function render(): Promise<VueWrapper> {
  const store = useProjectComponentsStore()
  vi.mocked(api.fetchProjectComponents).mockResolvedValue([saved])
  await store.load(PROJECT)
  wrapper = mount(ProjectComponentCard, {
    props: { component: saved, position: 0, canMoveUp: false, canMoveDown: false, root: null },
    global: { plugins: [] }
  })
  return wrapper
}

beforeEach(() => {
  vi.useFakeTimers()
  setActivePinia(createPinia())
  vi.resetAllMocks()
  vi.mocked(api.updateProjectComponent).mockResolvedValue({ ...saved, name: 'Gateway' })
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  vi.useRealTimers()
})

describe('a component card', () => {
  it('shows the saved name, folder and description, numbered from one', async () => {
    const card = await render()

    expect((card.get('[data-testid="component-name"]').element as HTMLInputElement).value).toBe('API')
    expect((card.get('[data-testid="component-folder"]').element as HTMLInputElement).value).toBe('/work/api')
    expect((card.get('[data-testid="component-description"]').element as HTMLTextAreaElement).value).toBe('The REST surface.')
    expect(card.text()).toContain('01')
  })

  it('saves only what changed, after a pause in typing', async () => {
    const card = await render()

    await card.get('[data-testid="component-name"]').setValue('Gateway')
    expect(api.updateProjectComponent).not.toHaveBeenCalled()
    await vi.advanceTimersByTimeAsync(700)

    expect(api.updateProjectComponent).toHaveBeenCalledOnce()
    expect(api.updateProjectComponent).toHaveBeenCalledWith('c-1', { name: 'Gateway' })
  })

  it('saves when the field is left, without waiting out the pause', async () => {
    const card = await render()

    await card.get('[data-testid="component-description"]').setValue('Routes requests.')
    await card.get('[data-testid="component-description"]').trigger('blur')

    expect(api.updateProjectComponent).toHaveBeenCalledWith('c-1', { description: 'Routes requests.' })
  })

  it('never saves a blank name or folder', async () => {
    const card = await render()

    await card.get('[data-testid="component-name"]').setValue('   ')
    await card.get('[data-testid="component-folder"]').setValue('')
    await card.get('[data-testid="component-folder"]').trigger('blur')
    await vi.advanceTimersByTimeAsync(700)

    expect(api.updateProjectComponent).not.toHaveBeenCalled()
  })

  it('marks the fields invalid when the server refuses, and clears the mark on the next keystroke', async () => {
    vi.mocked(api.updateProjectComponent).mockRejectedValue(new Error('This project already has a component named Gateway'))
    const card = await render()

    await card.get('[data-testid="component-name"]').setValue('Gateway')
    await card.get('[data-testid="component-name"]').trigger('blur')
    await flushPromises()
    expect(card.get('[data-testid="component-name"]').attributes('aria-invalid')).toBe('true')
    expect((card.get('[data-testid="component-name"]').element as HTMLInputElement).value).toBe('Gateway')

    await card.get('[data-testid="component-name"]').setValue('Gateway 2')
    expect(card.get('[data-testid="component-name"]').attributes('aria-invalid')).toBeUndefined()
  })

  it('saves a pending edit when it goes away while still listed', async () => {
    const card = await render()
    await card.get('[data-testid="component-name"]').setValue('Gateway')

    card.unmount()
    wrapper = null
    await flushPromises()

    expect(api.updateProjectComponent).toHaveBeenCalledWith('c-1', { name: 'Gateway' })
  })

  it('drops a pending edit when the component was deleted from under it', async () => {
    const card = await render()
    await card.get('[data-testid="component-name"]').setValue('Gateway')
    vi.mocked(api.deleteProjectComponent).mockResolvedValue()
    await useProjectComponentsStore().remove(PROJECT, saved.id)

    card.unmount()
    wrapper = null
    await vi.advanceTimersByTimeAsync(1000)

    expect(api.updateProjectComponent).not.toHaveBeenCalled()
  })

  it('reports a move or a delete to the list that owns it', async () => {
    const card = await render()
    await card.setProps({ canMoveDown: true })

    await card.get('[data-testid="component-down"]').trigger('click')
    await card.get('[data-testid="component-delete"]').trigger('click')

    expect(card.emitted('move')).toEqual([['down']])
    expect(card.emitted('delete')).toHaveLength(1)
  })
})
